#!/usr/bin/env python3
"""Second-stage tree-shaking for AOT archives.

The Windows first stage in ``windows/msvc/build_template.py`` links
representative executables with ``lld-link /verbose`` and removes every archive
member the linker does not mention.  That keeps too much on some targets
(notably AArch64): the verbose output only contains member basenames, so two
members such as ``...\\objr\\arm64\\foo.obj`` and
``...\\objr\\arm64ec\\foo.obj`` look identical and both are kept.

This script does not trust the verbose output.  It uses the linker itself as an
oracle:

* link the archive into every representative executable (GUI/console entry
  point and each virgl variant, or a caller-supplied probe command);
* try to delete one object member;
* if all links still succeed, commit the deletion and re-test previously kept
  members, because the deletion may have made them redundant;
* otherwise keep the member;
* continue until the candidate queue is empty.

The result is a 1-minimal archive: no remaining object can be removed on its
own without breaking one of the probe links.  Import-library members (``.dll``)
are never touched.

Objects are tested one at a time: batching several deletions into a single
trial made the search slower on real archives, because a failed batch still has
to be split down to individual members.  Independent single-member trials are
parallelized instead: a round tests up to ``jobs`` candidates concurrently
against the same archive, then commits the survivors and re-links the probes
once to verify the combination.  Removal is not monotone (dropping two members
can break a link that either removal alone leaves intact), so a round that
fails verification is rolled back and replayed one member at a time.

To keep a trial cheap, the archive is temporarily split while it is minimized:
the unchanged import members are copied once into their own archive, and the
object members are cut into contiguous slices that are linked in the original
member order.  A trial then copies and rewrites only the slice that holds its
candidate.  The final archive is rebuilt from the original by deleting the
removed members, so member order and duplicate names are preserved exactly.  If
the split archives do not link like the original, the run falls back to the
unsplit archive.

``jobs`` is the logical CPU count plus two, as Ninja does, or 6 when the count
cannot be determined.

The module is importable.  ``windows/msvc/build_template.py`` does::

    import tree_shake

    tree_shake.tree_shake(
        archive=webrogue_aot_lib_path,
        objects=[
            os.path.join(out_dir, "gui.obj"),
            os.path.join(out_dir, "console.obj"),
        ],
        output=webrogue_aot_lib_out_path,
        machine=win_arch,
        app_obj=os.path.join(template_dir, f"empty-{rust_arch}.obj"),
        virgl_libs=[
            os.path.join(out_dir, "webrogue_virgl_lib_impl.a"),
            os.path.join(out_dir, "webrogue_virgl_lib_stub.a"),
        ],
    )

Linux builds pass the exact final ``ld.lld`` command as a probe template with
``{archive}`` and ``{output}`` placeholders::

    python3 scripts/tree_shake.py libwebrogue_aot_lib.a \\
        --output libwebrogue_aot_lib.a \\
        --machine x86_64 \\
        --probe-command 'ld.lld ... -o {output} ... {archive} ...'

Command line usage::

    python3 scripts/tree_shake.py <archive> \\
        --object gui.obj --object console.obj --output out.lib

``--object`` is required for the default MSVC probes; ``--probe-command``
replaces them entirely.  The output defaults to the input path and is replaced
atomically after verification.
"""

from __future__ import annotations

import argparse
import ntpath
import os
import shlex
import shutil
import subprocess
import sys
import tempfile
from collections import Counter, deque
from concurrent.futures import Future, ThreadPoolExecutor
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable, Sequence

MACHINES = {
    "x86_64": "x64",
    "amd64": "x64",
    "x64": "x64",
    "aarch64": "arm64",
    "arm64": "arm64",
}
ARCH_NAMES = {"x64": "x86_64", "arm64": "aarch64"}

# Windows limits a command line to 32767 characters.  Keep some headroom for
# the executable name and quoting.
MAX_AR_COMMAND_CHARS = 24000

# While minimizing, the archive is cut into this many slices so that a trial
# only has to copy and rewrite the slice holding the member under test.  The
# slices are recombined into a single archive at the end.
SPLIT_PARTS = 16


def default_jobs() -> int:
    """Number of trials to run at once: logical CPUs + 2, as Ninja does."""
    try:
        count = len(os.sched_getaffinity(0))
    except (AttributeError, OSError):
        count = os.cpu_count()
    return count + 2 if count else 6


class TreeShakeError(RuntimeError):
    """Raised when tree-shaking cannot be completed."""


@dataclass
class TreeShakeResult:
    archive: Path
    output: Path
    total_members: int
    object_members: int
    removed_members: int
    removed: list[str]
    input_size: int
    output_size: int


@dataclass(frozen=True)
class _Probe:
    """One representative executable that the archive must keep linking."""

    name: str
    machine: str
    objects: tuple[Path, ...]
    libraries: tuple[Path, ...]
    workdir: Path
    command: tuple[str, ...] | None = None

    def link(
        self,
        archives: Sequence[Path],
        *,
        verbose: bool = False,
        tag: str | None = None,
    ):
        suffix = f"-{tag}" if tag else ""
        output = self.workdir / f"probe-{self.name}{suffix}.exe"
        if self.command is not None:
            cmd: list[str] = []
            for part in self.command:
                if part == "{archive}":
                    cmd.extend(str(archive) for archive in archives)
                    continue
                if "{archive}" in part and len(archives) > 1:
                    raise TreeShakeError(
                        "{archive} must be a standalone argument when the "
                        "archive is split into several link inputs"
                    )
                cmd.append(
                    part.replace("{archive}", str(archives[0])).replace(
                        "{output}", str(output)
                    )
                )
            return _run(cmd, capture=True, check=False, verbose=verbose)
        cmd = [
            "lld-link",
            f"-out:{output}",
            "-nologo",
            f"-machine:{self.machine}",
            *self.objects,
            *archives,
            *self.libraries,
            "/nodefaultlib",
        ]
        return _run(cmd, capture=True, check=False, verbose=verbose)


class _Reporter:
    def __init__(self, total_candidates: int, verbose: bool):
        self.total_candidates = total_candidates
        self.verbose = verbose
        self.trials = 0
        self.removed = 0
        self.kept_count = 0
        self._next_progress = 50

    def begin(self) -> None:
        self.trials += 1
        if self.trials >= self._next_progress:
            print(
                f"  progress: {self.trials} trials, {self.removed} removed, "
                f"{self.kept_count} kept, {self.total_candidates} initial "
                "candidates",
                file=sys.stderr,
            )
            self._next_progress += 50

    def failed(self, member: str, probe: _Probe) -> None:
        if self.verbose:
            print(
                f"  trial: cannot remove {member}: {probe.name} failed",
                file=sys.stderr,
            )

    def kept_member(self, member: str) -> None:
        self.kept_count += 1
        if self.verbose:
            print(f"  keep: {member}", file=sys.stderr)

    def committed(self, member: str) -> None:
        self.removed += 1
        if self.verbose:
            print(f"  removed: {member}", file=sys.stderr)


@dataclass(frozen=True)
class _Split:
    """Link inputs that stand in for the archive while it is minimized.

    Import-library members never change, so they go into one trailing archive.
    The object members are cut into contiguous slices that are linked in the
    original member order.  A trial then only copies and rewrites the slice
    holding the member under test.  ``tree_shake`` recombines the result by
    deleting the removed members from the original archive, which preserves
    member order and duplicate names exactly.
    """

    archives: tuple[Path, ...]
    chunk_of: dict[str, int]


def _splittable(probes: Sequence[_Probe]) -> bool:
    """Whether every probe takes several archives in place of ``{archive}``."""
    for probe in probes:
        for part in probe.command or ():
            if "{archive}" in part and part != "{archive}":
                return False
    return True


def _split_archive(
    archive: Path,
    members: Sequence[str],
    directory: Path,
    parts: int,
) -> _Split | None:
    """Cut ``archive`` into link inputs that a trial can rewrite cheaply.

    The import-library members never change, so they are copied once into a
    trailing archive.  The object members are cut into ``parts`` contiguous
    slices, which are linked in the original member order.  Returns ``None``
    when splitting would not pay off.

    Separate archives are equivalent to the single one here: each slice is a
    real archive, and both lld drivers resolve archive members lazily from a
    global symbol table, so a member in one slice can pull one from another.
    Keeping the slices contiguous preserves the first-definition choice for
    duplicate symbols.  A linker that scans archives in order instead (GNU ld)
    would need ``--start-group``; the baseline link catches that and the caller
    falls back to the unsplit archive.
    """
    names = list(dict.fromkeys(members))
    objects = [name for name in names if not is_import_member(name)]
    imports = [name for name in names if is_import_member(name)]
    if not imports or len(objects) < 2 * parts:
        return None

    directory.mkdir(parents=True, exist_ok=True)
    counts = Counter(members)

    fixed = directory / "fixed.lib"
    shutil.copyfile(archive, fixed)
    delete_members(
        fixed, objects, counts=counts, verbose=False, max_items=2048
    )

    # Slices are produced by halving, so each archive is rewritten a handful
    # of times while it shrinks instead of once per slice at full size.
    root = directory / "root.lib"
    shutil.copyfile(archive, root)
    delete_members(
        root, imports, counts=counts, verbose=False, max_items=2048
    )

    chunk_of: dict[str, int] = {}
    archives = _halve(root, objects, counts, directory, parts, [0], chunk_of)
    archives.append(fixed)
    return _Split(archives=tuple(archives), chunk_of=chunk_of)


def _halve(
    source: Path,
    names: Sequence[str],
    counts: Counter[str],
    directory: Path,
    parts: int,
    counter: list[int],
    chunk_of: dict[str, int],
) -> list[Path]:
    """Split ``source`` (holding ``names``) into ``parts`` slices."""
    if parts <= 1 or len(names) < 2:
        index = counter[0]
        counter[0] = index + 1
        path = directory / f"chunk-{index}.lib"
        os.replace(source, path)
        for name in names:
            chunk_of[name] = index
        return [path]

    middle = len(names) // 2
    halves = [list(names[:middle]), list(names[middle:])]
    paths = [
        source.with_name(f"{source.name}-{suffix}") for suffix in ("a", "b")
    ]
    for path in paths:
        shutil.copyfile(source, path)
    source.unlink()
    # Duplicate names live in one slice: llvm-ar deletes every copy at once.
    for index, path in enumerate(paths):
        delete_members(
            path,
            halves[1 - index],
            counts=counts,
            verbose=False,
            max_items=2048,
        )

    return _halve(
        paths[0], halves[0], counts, directory, parts // 2, counter, chunk_of
    ) + _halve(
        paths[1],
        halves[1],
        counts,
        directory,
        parts - parts // 2,
        counter,
        chunk_of,
    )


class _Minimizer:
    """Greedy delta-debugging of an archive using the probe links as oracle.

    Trials are independent as long as they start from the same archive, so a
    round tests up to ``jobs`` candidates concurrently against a frozen copy of
    the working archive.  Commits happen afterwards, in candidate order, which
    keeps the result reproducible.
    """

    def __init__(
        self,
        working: Path,
        probes: Sequence[_Probe],
        reporter: _Reporter,
        verbose: bool,
        jobs: int,
        multiplicity: Counter[str],
        split: _Split | None = None,
    ):
        self.working = working
        self.probes = probes
        self.reporter = reporter
        self.verbose = verbose
        self.jobs = jobs
        # Each concurrent trial needs its own archive copy and probe output.
        self.trials = [
            working.with_name(f"{working.name}.trial-{slot}")
            for slot in range(jobs)
        ]
        self.archives = list(split.archives) if split else [working]
        self.chunk_of = (
            dict(split.chunk_of)
            if split
            else {member: 0 for member in multiplicity}
        )
        self.chunk_counts = [len(list_members(path)) for path in self.archives]
        self.multiplicity = multiplicity
        self.removed: list[str] = []

    def run(self, candidates: Sequence[str]) -> None:
        queue = deque(candidates)
        kept: list[str] = []
        with ThreadPoolExecutor(max_workers=self.jobs) as pool:
            while queue:
                batch = [
                    queue.popleft() for _ in range(min(self.jobs, len(queue)))
                ]
                futures = self._submit(pool, batch)
                successes: list[str] = []
                for member, future in zip(batch, futures):
                    if future.result():
                        successes.append(member)
                    else:
                        self.reporter.kept_member(member)
                        kept.append(member)
                if not successes:
                    continue
                applied, rejected = self._commit(pool, successes)
                for member in applied:
                    self.reporter.committed(member)
                for member in rejected:
                    self.reporter.kept_member(member)
                    kept.append(member)
                # A removal can make previously kept members redundant.
                queue.extend(kept)
                kept.clear()

    def _submit(
        self, pool: ThreadPoolExecutor, batch: Sequence[str]
    ) -> list[Future[bool]]:
        futures = []
        for slot, member in enumerate(batch):
            self.reporter.begin()
            futures.append(
                pool.submit(self._try_remove, member, slot, f"t{slot}")
            )
        return futures

    def _try_remove(self, member: str, slot: int, tag: str) -> bool:
        # ``chunk_counts`` is only updated by the coordinating thread between
        # rounds, so every trial in a round sees the same baseline.
        chunk = self.chunk_of[member]
        trial = self.trials[slot]
        shutil.copyfile(self.archives[chunk], trial)
        delete_members(
            trial,
            [member],
            counts=self.multiplicity,
            verbose=self.verbose,
        )
        archives = [
            trial if index == chunk else path
            for index, path in enumerate(self.archives)
        ]

        for probe in self.probes:
            result = probe.link(archives, verbose=self.verbose, tag=tag)
            if result.returncode:
                self.reporter.failed(member, probe)
                if self.verbose:
                    print(_link_failure(probe, result), file=sys.stderr)
                return False

        # llvm-ar silently ignores names that do not match a member.  Make sure
        # a successful link is not just a link of the unmodified archive.
        actual = len(list_members(trial))
        expected = self.chunk_counts[chunk] - self.multiplicity[member]
        if actual != expected:
            raise TreeShakeError(
                f"llvm-ar removed {self.chunk_counts[chunk] - actual} members "
                f"but {self.multiplicity[member]} were requested for "
                f"{member}; refusing to commit an incomplete deletion"
            )
        return True

    def _commit(
        self, pool: ThreadPoolExecutor, members: Sequence[str]
    ) -> tuple[list[str], list[str]]:
        """Apply individually verified removals and re-check the combination.

        Returns the members that ended up removed and the ones that had to be
        kept because they were only safe on their own.
        """
        if len(members) == 1:
            self._apply(members)
            return list(members), []

        touched = {self.chunk_of[member] for member in members}
        backups = {}
        for chunk in touched:
            path = self.archives[chunk]
            backup = path.with_name(path.name + ".backup")
            shutil.copyfile(path, backup)
            backups[chunk] = (backup, self.chunk_counts[chunk])
        base_removed = len(self.removed)
        self._apply(members)
        if self._verify(pool):
            for backup, _ in backups.values():
                backup.unlink()
            return list(members), []

        # Link success is not monotone: two individually safe removals can
        # break the link together.  Restore the round's starting point and
        # re-apply them one at a time, which is always sound.
        print(
            f"  warning: {len(members)} combined removals failed "
            "verification; re-applying them one at a time",
            file=sys.stderr,
        )
        for chunk, (backup, count) in backups.items():
            os.replace(backup, self.archives[chunk])
            self.chunk_counts[chunk] = count
        del self.removed[base_removed:]
        applied: list[str] = []
        rejected: list[str] = []
        for member in members:
            self.reporter.begin()
            if self._try_remove(member, 0, "serial"):
                self._apply([member])
                applied.append(member)
            else:
                rejected.append(member)
        return applied, rejected

    def _apply(self, members: Sequence[str]) -> None:
        by_chunk: dict[int, list[str]] = {}
        for member in members:
            by_chunk.setdefault(self.chunk_of[member], []).append(member)
        for chunk, names in by_chunk.items():
            delete_members(
                self.archives[chunk],
                names,
                counts=self.multiplicity,
                verbose=self.verbose,
            )
            removed = sum(self.multiplicity[name] for name in names)
            expected = self.chunk_counts[chunk] - removed
            actual = len(list_members(self.archives[chunk]))
            if actual != expected:
                raise TreeShakeError(
                    f"llvm-ar removed {self.chunk_counts[chunk] - actual} "
                    f"members but {removed} were requested; "
                    "refusing to commit an incomplete deletion"
                )
            self.chunk_counts[chunk] = actual
        self.removed.extend(members)

    def _verify(self, pool: ThreadPoolExecutor) -> bool:
        futures = [
            pool.submit(
                probe.link, self.archives, verbose=self.verbose, tag="verify"
            )
            for probe in self.probes
        ]
        results = [future.result() for future in futures]
        return all(not result.returncode for result in results)

    def cleanup(self) -> None:
        leftovers = [
            *self.trials,
            *(path.with_name(path.name + ".backup") for path in self.archives),
        ]
        for path in leftovers:
            if path.exists():
                path.unlink()


def _format_cmd(cmd: Sequence[object]) -> str:
    parts = [str(part) for part in cmd]
    if os.name == "nt":
        return subprocess.list2cmdline(parts)
    return shlex.join(parts)


def _run(
    cmd: Sequence[object],
    *,
    capture: bool = False,
    check: bool = True,
    verbose: bool = False,
    cwd: Path | None = None,
):
    parts = [str(part) for part in cmd]
    if verbose:
        print("+ " + _format_cmd(parts), file=sys.stderr)
    try:
        result = subprocess.run(
            parts,
            check=False,
            cwd=cwd,
            stdout=subprocess.PIPE if capture else None,
            stderr=subprocess.PIPE if capture else None,
            text=True,
            encoding="utf-8",
            errors="replace",
        )
    except OSError as exc:
        raise TreeShakeError(f"failed to run {parts[0]}: {exc}") from exc

    if check and result.returncode:
        detail = (result.stderr or result.stdout or "").strip()
        message = (
            f"command failed with exit code {result.returncode}: "
            f"{_format_cmd(parts)}"
        )
        if detail:
            message += f"\n{detail}"
        raise TreeShakeError(message)
    return result


def command_line_chunks(
    items: Sequence[str],
    max_items: int,
    max_chars: int = MAX_AR_COMMAND_CHARS,
) -> Iterable[list[str]]:
    """Split member names so a command line stays below the Windows limit."""
    chunk: list[str] = []
    length = 0
    for item in items:
        item_length = len(item) + 1
        if chunk and (len(chunk) >= max_items or length + item_length > max_chars):
            yield chunk
            chunk = []
            length = 0
        chunk.append(item)
        length += item_length
    if chunk:
        yield chunk


def list_members(archive: Path) -> list[str]:
    result = _run(["llvm-ar", "t", archive], capture=True)
    return [line for line in result.stdout.splitlines() if line]


def delete_members(
    archive: Path,
    members: Sequence[str],
    *,
    counts: Counter[str] | None = None,
    verbose: bool,
    max_items: int = 128,
) -> None:
    # llvm-ar deletes one copy per occurrence of a member name.  Archives can
    # contain duplicate names, so repeat the name for every copy we intend to
    # remove.
    expanded: list[str] = []
    for member in members:
        expanded.extend([member] * (counts[member] if counts else 1))
    for chunk in command_line_chunks(expanded, max_items):
        _run(["llvm-ar", "d", archive, *chunk], verbose=verbose)


def is_import_member(member: str) -> bool:
    return member.lower().endswith(".dll")


def member_basename(member: str) -> str:
    return ntpath.basename(member)


def member_directory(member: str) -> str:
    return ntpath.basename(ntpath.dirname(member))


def variant_name(library: Path) -> str:
    name = library.stem
    for prefix in ("libwebrogue_virgl_lib_", "webrogue_virgl_lib_"):
        if name.startswith(prefix):
            return name[len(prefix) :] or library.stem
    return name


def _sanitize(name: str) -> str:
    return "".join(
        char if char.isalnum() or char in "-_." else "_" for char in name
    )


def _link_failure(probe: _Probe, result) -> str:
    text = (result.stderr or result.stdout or "").strip()
    if len(text) > 4000:
        text = "...\n" + text[-4000:]
    return f"probe {probe.name} failed (exit {result.returncode}):\n{text}"


def detect_machine(archive: Path) -> str:
    text = str(archive).lower().replace("\\", "/")
    if "aarch64" in text or "arm64" in text:
        return "arm64"
    return "x64"


def resolve_machine(value: str) -> str:
    machine = MACHINES.get(value.lower())
    if machine is None:
        raise TreeShakeError(
            f"unsupported machine {value!r}; "
            f"expected one of {', '.join(sorted(MACHINES))}"
        )
    return machine


def find_app_object(archive: Path, machine: str) -> Path | None:
    arch = ARCH_NAMES[machine]
    candidate = archive.parent / f"empty-{arch}.obj"
    return candidate if candidate.is_file() else None


def find_virgl_libraries(archive: Path) -> list[Path]:
    libraries: list[Path] = []
    for pattern in (
        "webrogue_virgl_lib_*.a",
        "webrogue_virgl_lib_*.lib",
        "libwebrogue_virgl_lib_*.a",
    ):
        libraries.extend(sorted(archive.parent.glob(pattern)))
    # Keep a stable order, but do not link the same file twice.
    return sorted(set(libraries))


def human_size(size: int) -> str:
    value = float(size)
    for unit in ("B", "KiB", "MiB", "GiB"):
        if value < 1024 or unit == "GiB":
            return f"{value:.1f} {unit}" if unit != "B" else f"{size} B"
        value /= 1024
    return f"{size} B"


def _resolve_file(path: str | os.PathLike[str], label: str) -> Path:
    result = Path(path).expanduser().resolve()
    if not result.is_file():
        raise TreeShakeError(f"{label} does not exist: {result}")
    return result


def _build_probes(
    entry_objects: Sequence[Path],
    app_obj: Path | None,
    virgl_libraries: Sequence[Path],
    link_inputs: Sequence[Path],
    machine: str,
    workdir: Path,
) -> list[_Probe]:
    object_inputs = [
        path for path in link_inputs if path.suffix.lower() in (".obj", ".o")
    ]
    library_inputs = [path for path in link_inputs if path not in object_inputs]

    variants: list[list[Path]] = (
        [[library] for library in virgl_libraries] if virgl_libraries else [[]]
    )

    probes: list[_Probe] = []
    used_names: set[str] = set()
    # Variant-major order: the impl variants usually reference more symbols, so
    # a failed removal is detected with fewer links.
    for variant_index, variant in enumerate(variants):
        label = variant_name(variant[0]) if variant else "default"
        for entry_index, entry_obj in enumerate(entry_objects):
            probe_objects = tuple(
                path
                for path in (app_obj, entry_obj, *object_inputs)
                if path is not None
            )
            libraries = tuple([*variant, *library_inputs])
            entry_name = _sanitize(entry_obj.stem) or f"object-{entry_index + 1}"
            name = _sanitize(f"{entry_name}-{label}")
            if name in used_names:
                name = f"{name}-{entry_index + 1}"
            used_names.add(name)
            probes.append(
                _Probe(
                    name=name,
                    machine=machine,
                    objects=probe_objects,
                    libraries=libraries,
                    workdir=workdir,
                )
            )
    return probes


def _build_command_probes(
    commands: Sequence[Sequence[str]],
    machine: str,
    workdir: Path,
) -> list[_Probe]:
    """Build probes from complete linker command templates.

    Each command must contain ``{archive}`` and ``{output}`` placeholders.
    This is how non-MSVC builds reuse this script without duplicating it.
    """
    probes: list[_Probe] = []
    for index, command in enumerate(commands, start=1):
        parts = tuple(str(part) for part in command)
        if not parts:
            raise TreeShakeError(f"probe command {index} is empty")
        if not any("{archive}" in part for part in parts):
            raise TreeShakeError(
                f"probe command {index} must contain {{archive}}"
            )
        if not any("{output}" in part for part in parts):
            raise TreeShakeError(
                f"probe command {index} must contain {{output}}"
            )
        probes.append(
            _Probe(
                name=f"cmd-{index}",
                machine=machine,
                objects=(),
                libraries=(),
                workdir=workdir,
                command=parts,
            )
        )
    return probes


def tree_shake(
    archive: str | os.PathLike[str],
    objects: Sequence[str | os.PathLike[str]] | None = None,
    output: str | os.PathLike[str] | None = None,
    *,
    machine: str | None = None,
    app_obj: str | os.PathLike[str] | None = None,
    virgl_libs: Sequence[str | os.PathLike[str]] | None = None,
    link_inputs: Sequence[str | os.PathLike[str]] = (),
    probe_commands: Sequence[Sequence[str]] | None = None,
    verbose: bool = False,
) -> TreeShakeResult:
    """Remove archive members that are not needed by any probe link.

    ``archive`` is the input.  ``objects`` is the list of entry-point objects
    (for example ``gui.obj`` and ``console.obj``); it is required unless
    ``probe_commands`` is given.  ``output`` defaults to the input path (in-place
    replacement).  The output is written atomically; on any error the previous
    output (and the input, if it is the same file) is left untouched.

    ``machine`` may be ``x64``/``arm64`` (or an alias such as ``x86_64``); it is
    detected from the archive path when omitted.  ``app_obj`` defaults to an
    ``empty-<arch>.obj`` next to the archive.  ``virgl_libs`` defaults to all
    ``webrogue_virgl_lib_*.a``/``.lib`` (and ``libwebrogue_virgl_lib_*.a``)
    files next to the archive; each one is linked as a separate probe because
    the impl and stub variants define the same symbols and cannot be linked
    together.

    ``probe_commands`` overrides the default MSVC probe construction.  Each
    command is a sequence of arguments with ``{archive}`` and ``{output}``
    placeholders; this is how the Linux builds pass their exact ``ld.lld``
    command without duplicating this script.
    """
    archive = _resolve_file(archive, "archive")
    object_paths: list[Path] = []
    if probe_commands is None:
        if not objects:
            raise TreeShakeError(
                "objects are required unless probe_commands is given"
            )
        object_paths = [
            _resolve_file(path, "entry object") for path in objects
        ]
    elif not probe_commands:
        raise TreeShakeError("probe_commands must not be empty")

    if output is not None:
        output = Path(output).expanduser().resolve()
    else:
        output = archive

    machine = resolve_machine(machine or detect_machine(archive))

    app_obj_path: Path | None = None
    virgl_paths: list[Path] = []
    link_paths: list[Path] = []
    if probe_commands is None:
        if app_obj is not None:
            app_obj_path = _resolve_file(app_obj, "app object")
        else:
            app_obj_path = find_app_object(archive, machine)

        if virgl_libs is not None:
            virgl_paths = [
                _resolve_file(path, "virgl library") for path in virgl_libs
            ]
        else:
            virgl_paths = find_virgl_libraries(archive)

        link_paths = [
            _resolve_file(path, "link input") for path in link_inputs
        ]

    required_tools = ["llvm-ar"]
    if probe_commands is None:
        required_tools.append("lld-link")
    else:
        required_tools.extend(
            command[0] for command in probe_commands if command
        )
    for tool in dict.fromkeys(required_tools):
        if shutil.which(tool) is None:
            raise TreeShakeError(f"{tool} was not found in PATH")

    output.parent.mkdir(parents=True, exist_ok=True)

    members = list_members(archive)
    multiplicity = Counter(members)
    object_members = [
        member for member in members if not is_import_member(member)
    ]
    # Duplicate member names are a single candidate: llvm-ar can remove all
    # copies together, and the link oracle decides whether that is safe.
    candidates: list[str] = []
    seen: set[str] = set()
    for member in object_members:
        if member not in seen:
            seen.add(member)
            candidates.append(member)
    if len(candidates) != len(object_members):
        print(
            f"warning: {len(object_members) - len(candidates)} duplicate "
            "object member name(s) treated as one candidate each",
            file=sys.stderr,
        )
    if not candidates:
        raise TreeShakeError("the archive contains no object members")

    # Keep members from the same directory together.  The order does not affect
    # correctness, but it makes progress and removal logs easier to follow.
    candidates.sort(
        key=lambda member: (
            member_directory(member).lower(),
            member_basename(member).lower(),
            member,
        )
    )

    input_size = archive.stat().st_size
    jobs = default_jobs()
    print(
        f"archive: {archive} ({human_size(input_size)}, {len(members)} "
        f"members, {len(object_members)} objects)"
    )
    print(f"machine: {machine}")
    print(f"jobs: {jobs}")

    work_path = output.parent / f".{output.name}.tree-shake-{os.getpid()}.tmp"
    minimizer: _Minimizer | None = None

    with tempfile.TemporaryDirectory(prefix="webrogue-tree-shake-") as tmp:
        probe_dir = Path(tmp)
        if probe_commands is None:
            probes = _build_probes(
                object_paths,
                app_obj_path,
                virgl_paths,
                link_paths,
                machine,
                probe_dir,
            )
        else:
            probes = _build_command_probes(
                probe_commands,
                machine,
                probe_dir,
            )
        print("probes: " + ", ".join(probe.name for probe in probes))

        def baseline_failure(archives: Sequence[Path]):
            for probe in probes:
                result = probe.link(archives, verbose=verbose)
                if result.returncode:
                    return probe, result
            return None

        split: _Split | None = None
        if _splittable(probes):
            try:
                split = _split_archive(
                    archive, members, probe_dir / "split", SPLIT_PARTS
                )
            except TreeShakeError as exc:
                print(
                    f"warning: cannot split the archive ({exc}); "
                    "minimizing it in one piece",
                    file=sys.stderr,
                )
        baseline_archives = list(split.archives) if split else [archive]
        failure = baseline_failure(baseline_archives)
        if failure is not None and split is not None:
            print(
                "warning: the split archive does not link like the original; "
                "falling back to the unsplit archive",
                file=sys.stderr,
            )
            split = None
            baseline_archives = [archive]
            failure = baseline_failure(baseline_archives)
        if failure is not None:
            probe, result = failure
            hint = (
                "\nIf a symbol is missing, pass --app-obj and/or "
                "--virgl-lib."
                if probe_commands is None
                else ""
            )
            raise TreeShakeError(
                "baseline link failed; the input archive does not link "
                "with the selected probes.\n"
                + _link_failure(probe, result)
                + hint
            )
        print(f"baseline: all {len(probes)} probe link(s) succeeded")
        if split is not None:
            print(
                f"split: {len(split.archives) - 1} object slice(s) plus the "
                "unchanged import members"
            )

        try:
            if split is None:
                # Without slices the minimizer rewrites the working copy.
                shutil.copyfile(archive, work_path)
            reporter = _Reporter(len(candidates), verbose)
            minimizer = _Minimizer(
                work_path,
                probes,
                reporter,
                verbose,
                jobs,
                multiplicity,
                split,
            )

            print(
                f"testing {len(candidates)} object member(s)",
                file=sys.stderr,
            )
            minimizer.run(candidates)

            # Recombine: rebuild the output from the original archive so the
            # slices used during minimization never leak into the result.
            shutil.copyfile(archive, work_path)
            delete_members(
                work_path,
                minimizer.removed,
                counts=multiplicity,
                verbose=verbose,
            )

            for probe in probes:
                result = probe.link([work_path], verbose=verbose)
                if result.returncode:
                    raise TreeShakeError(
                        "internal error: the minimized archive failed "
                        "verification.\n"
                        + _link_failure(probe, result)
                    )
            print("verification: all probes still link")

            if output == archive:
                print(f"replacing input archive in place: {output}")
            try:
                os.replace(work_path, output)
            except OSError as exc:
                raise TreeShakeError(
                    f"failed to install {output}: {exc}"
                ) from exc
        finally:
            if minimizer is not None:
                minimizer.cleanup()
            if work_path.exists():
                work_path.unlink()

    output_members = list_members(output)
    output_size = output.stat().st_size
    import_members = len(members) - len(object_members)
    kept_objects = len(output_members) - import_members
    removed_members = len(members) - len(output_members)

    print(
        f"wrote {output} ({human_size(output_size)}, "
        f"{len(output_members)} members); removed {removed_members} "
        f"member(s), kept {kept_objects} object(s)"
    )

    return TreeShakeResult(
        archive=archive,
        output=output,
        total_members=len(members),
        object_members=len(object_members),
        removed_members=removed_members,
        removed=list(minimizer.removed) if minimizer is not None else [],
        input_size=input_size,
        output_size=output_size,
    )


def parse_args(argv: Sequence[str] | None = None):
    parser = argparse.ArgumentParser(
        description=(
            "Second-stage tree-shaking: remove archive members whose removal "
            "still lets every representative executable link."
        ),
    )
    parser.add_argument(
        "archive",
        type=Path,
        help="archive to tree-shake",
    )
    parser.add_argument(
        "--object",
        dest="objects",
        type=Path,
        action="append",
        default=None,
        help=(
            "entry-point object, repeatable; required unless "
            "--probe-command is given"
        ),
    )
    parser.add_argument(
        "--output",
        type=Path,
        default=None,
        help="tree-shaken archive to write (default: the input path)",
    )
    parser.add_argument(
        "--machine",
        default=None,
        help="linker machine, x64 or arm64 (default: detect from archive path)",
    )
    parser.add_argument(
        "--app-obj",
        type=Path,
        default=None,
        help=(
            "compiled AOT payload defining WEBROGUE_AOT "
            "(default: empty-<arch>.obj next to the archive)"
        ),
    )
    parser.add_argument(
        "--virgl-lib",
        type=Path,
        action="append",
        default=None,
        help=(
            "virgl library to link, repeatable "
            "(default: *webrogue_virgl_lib_*.a/.lib next to the archive)"
        ),
    )
    parser.add_argument(
        "--link-input",
        type=Path,
        action="append",
        default=[],
        help="extra object or library linked into every probe, repeatable",
    )
    parser.add_argument(
        "--probe-command",
        action="append",
        default=None,
        metavar="COMMAND",
        help=(
            "complete linker command to use as a probe, repeatable; must "
            "contain {archive} and {output} placeholders (for example an "
            "ld.lld command on Linux)"
        ),
    )
    parser.add_argument(
        "--log-removed",
        type=Path,
        default=None,
        help="write the names of removed members to this file",
    )
    parser.add_argument(
        "--verbose",
        "-v",
        action="store_true",
        help="print every tool invocation and every kept member",
    )
    return parser.parse_args(argv)


def main(argv: Sequence[str] | None = None) -> int:
    args = parse_args(argv)

    archive = args.archive.expanduser().resolve()
    probe_commands = (
        [shlex.split(command) for command in args.probe_command]
        if args.probe_command
        else None
    )
    try:
        result = tree_shake(
            archive=archive,
            objects=args.objects,
            output=args.output,
            machine=args.machine,
            app_obj=args.app_obj,
            virgl_libs=args.virgl_lib,
            link_inputs=args.link_input,
            probe_commands=probe_commands,
            verbose=args.verbose,
        )
    except (TreeShakeError, OSError) as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 1

    if args.log_removed is not None:
        log_path = args.log_removed.expanduser().resolve()
        log_path.parent.mkdir(parents=True, exist_ok=True)
        log_path.write_text(
            "\n".join(result.removed) + ("\n" if result.removed else ""),
            encoding="utf-8",
        )
        print(f"wrote removed-member list to {log_path}")

    return 0


if __name__ == "__main__":
    sys.exit(main())
