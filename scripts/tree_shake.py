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
are never touched.  Objects are tested one at a time: batching them made the
search slower on real archives because a failed batch still has to be split
down to individual members.

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

    def link(self, archive: Path, *, verbose: bool = False):
        output = self.workdir / f"probe-{self.name}.exe"
        if self.command is not None:
            cmd = [
                part.replace("{archive}", str(archive)).replace(
                    "{output}", str(output)
                )
                for part in self.command
            ]
            return _run(cmd, capture=True, check=False, verbose=verbose)
        cmd = [
            "lld-link",
            f"-out:{output}",
            "-nologo",
            f"-machine:{self.machine}",
            *self.objects,
            archive,
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
        elif self.removed == 1 or self.removed % 10 == 0:
            print(f"  removed {self.removed} member(s)", file=sys.stderr)


class _Minimizer:
    """Greedy delta-debugging of an archive using the probe links as oracle."""

    def __init__(
        self,
        working: Path,
        probes: Sequence[_Probe],
        reporter: _Reporter,
        verbose: bool,
    ):
        self.working = working
        self.probes = probes
        self.reporter = reporter
        self.verbose = verbose
        self.trial = working.with_name(working.name + ".trial")
        self.multiplicity = Counter(list_members(working))
        self.member_count = sum(self.multiplicity.values())
        self.removed: list[str] = []

    def run(self, candidates: Sequence[str]) -> None:
        queue = deque(candidates)
        kept: list[str] = []
        while queue:
            member = queue.popleft()
            if self._try_remove(member):
                self._commit(member)
                # A removal can make previously kept members redundant.
                queue.extend(kept)
                kept.clear()
            else:
                self.reporter.kept_member(member)
                kept.append(member)

    def _try_remove(self, member: str) -> bool:
        self.reporter.begin()
        shutil.copyfile(self.working, self.trial)
        delete_members(
            self.trial,
            [member],
            counts=self.multiplicity,
            verbose=self.verbose,
        )

        for probe in self.probes:
            result = probe.link(self.trial, verbose=self.verbose)
            if result.returncode:
                self.reporter.failed(member, probe)
                if self.verbose:
                    print(_link_failure(probe, result), file=sys.stderr)
                return False

        # llvm-ar silently ignores names that do not match a member.  Make sure
        # a successful link is not just a link of the unmodified archive.
        actual = len(list_members(self.trial))
        expected = self.member_count - self.multiplicity[member]
        if actual != expected:
            raise TreeShakeError(
                f"llvm-ar removed {self.member_count - actual} members but "
                f"{self.multiplicity[member]} were requested for {member}; "
                "refusing to commit an incomplete deletion"
            )
        return True

    def _commit(self, member: str) -> None:
        os.replace(self.trial, self.working)
        self.member_count -= self.multiplicity[member]
        self.removed.append(member)
        self.reporter.committed(member)

    def cleanup(self) -> None:
        if self.trial.exists():
            self.trial.unlink()


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
):
    parts = [str(part) for part in cmd]
    if verbose:
        print("+ " + _format_cmd(parts), file=sys.stderr)
    try:
        result = subprocess.run(
            parts,
            check=False,
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
) -> None:
    # llvm-ar deletes one copy per occurrence of a member name.  Archives can
    # contain duplicate names, so repeat the name for every copy we intend to
    # remove.
    expanded: list[str] = []
    for member in members:
        expanded.extend([member] * (counts[member] if counts else 1))
    for chunk in command_line_chunks(expanded, 128):
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
    print(
        f"archive: {archive} ({human_size(input_size)}, {len(members)} "
        f"members, {len(object_members)} objects)"
    )
    print(f"machine: {machine}")

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

        for probe in probes:
            result = probe.link(archive, verbose=verbose)
            if result.returncode:
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

        try:
            shutil.copyfile(archive, work_path)
            reporter = _Reporter(len(candidates), verbose)
            minimizer = _Minimizer(work_path, probes, reporter, verbose)

            print(
                f"testing {len(candidates)} object member(s)",
                file=sys.stderr,
            )
            minimizer.run(candidates)

            for probe in probes:
                result = probe.link(work_path, verbose=verbose)
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
