import os
import subprocess
import sys
import itertools

repo_dir = os.path.dirname(os.path.dirname(os.path.realpath(__file__)))

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
    subprocess.run(
        args,
        cwd=repo_dir
    ).check_returncode()


def get_all_subsets(arr):
    subsets = []
    # Loop from length 0 (empty set) to the full length of the array
    for i in range(len(arr) + 1):
        # Find all combinations of length i
        for comp in itertools.combinations(arr, i):
            subsets.append(list(comp))
    return subsets

check(package="webrogue")
for features in get_all_subsets(["run", "compile", "pack", "hub", "llvm"]):
    check(package="webrogue", features=features)

for ndk_target in ["arm64-v8a", "x86_64"]:
    for features in [["launcher"], ["runner"]]:
        check(package="webrogue-android", ndk_target=ndk_target, features=features)

# for features in [["launcher"], ["runner"]]:
#     check(package="webrogue-ios", features=features)