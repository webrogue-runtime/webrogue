import os
import subprocess
import sys
import itertools

try:
    import tqdm
    tqdm_func = tqdm.tqdm
except ImportError:
    def tqdm_func(x):
        return x

repo_dir = os.path.dirname(os.path.dirname(os.path.realpath(__file__)))

components = set(os.getenv("COMPONENTS", "all").split(","))
checked_components = set()

def is_component_selected(component):
    if "all" in components:
        checked_components.add("all")
        return True
    if component in components:
        checked_components.add(component)
    return component in components

pending_checks = []

def check(package=None, target=None, features=None, ndk_target=None):
    args = ["cargo"]
    if ndk_target:
        args.append("ndk")
        args.append("--target")
        args.append(ndk_target)
    args.append("check")
    if package:
        args.append("--package")
        args.append(package)
    if target:
        args.append("--target")
        args.append(target)
    if features:
        args.append("--features")
        args.append(",".join(features))
    args.append("--quiet")
    pending_checks.append(args)


def get_all_subsets(arr):
    subsets = []
    # Loop from length 0 (empty set) to the full length of the array
    for i in range(len(arr) + 1):
        # Find all combinations of length i
        for comp in itertools.combinations(arr, i):
            subsets.append(list(comp))
    return subsets

native_targets = []
if sys.platform == "win32":
    native_targets.append("x86_64-pc-windows-msvc")
elif sys.platform == "darwin":
    native_targets.extend(["x86_64-apple-darwin", "aarch64-apple-darwin"])
elif sys.platform == "linux":
    # Omit --target option for native Linux builds
    native_targets.append(None)
else:
    raise Exception(f"Unsupported platform: {sys.platform}")

if is_component_selected("cli"):
    for target in native_targets:
        check(package="webrogue", target=target)
        for features in get_all_subsets(["run", "compile", "pack", "hub", "llvm"]):
            check(package="webrogue", target=target, features=features)

if is_component_selected("aot-lib"):
    for target in native_targets:
        check(package="webrogue-aot-lib", target=target)
        for gfxstream_type in ["impl", "stub"]:
            check(package="webrogue-gfxstream-lib", target=target, features=[gfxstream_type])

if is_component_selected("android"):
    os.environ["CARGO_NDK_PLATFORM"] = open(os.path.join(repo_dir, "android", "android_api_version.txt")).read().strip()
    for ndk_target in ["arm64-v8a", "x86_64"]:
        for features in [["launcher"], ["runner"]]:
            check(package="webrogue-android", ndk_target=ndk_target, features=features)

# if is_component_selected("ios"):
#     for features in [["launcher"], ["runner"]]:
#         check(package="webrogue-ios", features=features)

for pending_check in tqdm_func(pending_checks):
    subprocess.run(pending_check, cwd=repo_dir, check=True)

if components != checked_components:
    raise Exception(f"Some specified components were not checked. Specified components :{components}")
