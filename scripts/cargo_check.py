import os
import subprocess
import sys

try:
    import tqdm
    tqdm_func = tqdm.tqdm
except ImportError:
    def tqdm_func(x):
        return x

repo_dir = os.path.dirname(os.path.dirname(os.path.realpath(__file__)))

components = set(os.getenv("COMPONENTS", "all").split(","))
checked_components = set()
android_api_version = open(os.path.join(repo_dir, "android", "android_api_version.txt")).read().strip()

def is_component_selected(component):
    if "all" in components:
        checked_components.add("all")
        return True
    if component in components:
        checked_components.add(component)
    return component in components

pending_checks = []

def check(package=None, target=None, features=None, ndk_target=None, no_default_features=False):
    args = ["cargo"]
    if ndk_target:
        args.append("ndk")
        args.append("--target")
        args.append(ndk_target)
        args.append("--platform")
        args.append(android_api_version)
    args.append("check")
    if package:
        args.append("--package")
        args.append(package)
    if target:
        args.append("--target")
        args.append(target)
    if no_default_features:
        args.append("--no-default-features")
    if features:
        args.append("--features")
        args.append(",".join(features))
    args.append("--locked")
    args.append("--quiet")
    pending_checks.append(args)


native_targets = []
if sys.platform == "win32":
    native_targets.append("x86_64-pc-windows-msvc")
elif sys.platform == "darwin":
    native_targets.append("aarch64-apple-darwin")
    # native_targets.append("x86_64-apple-darwin")
elif sys.platform == "linux":
    # Omit --target option for native Linux builds
    native_targets.append(None)
else:
    raise Exception(f"Unsupported platform: {sys.platform}")

if is_component_selected("cli"):
    for target in native_targets:
        check(
            package="webrogue",
            target=target,
            features=["run", "compile", "hub", "llvm"],
            no_default_features=True,
        )

if is_component_selected("aot-lib"):
    for target in native_targets:
        check(package="webrogue-aot-lib", target=target)
        for gfxstream_type in ["impl", "stub"]:
            check(package="webrogue-virgl-lib", target=target, features=[gfxstream_type])

if is_component_selected("android"):
    for ndk_target in ["arm64-v8a", "x86_64"]:
        for features in [["launcher"], ["runner"]]:
            check(package="webrogue-android", ndk_target=ndk_target, features=features)

if is_component_selected("ios"):
    for features in [["launcher"], ["runner"]]:
        check(package="webrogue-ios", features=features)

if is_component_selected("macos"):
    for features in [["runtime"], ["runner"]]:
        check(package="webrogue-macos", features=features)

for pending_check in tqdm_func(pending_checks):
    subprocess.run(
        pending_check,
        cwd=repo_dir,
        env={
            **os.environ,
            "CARGO_BUILD_WARNINGS": "deny"
        },
        check=True,
    )

if components != checked_components:
    raise Exception(f"Some specified components were not checked. Specified components :{components}")
