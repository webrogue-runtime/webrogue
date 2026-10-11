import json
import subprocess

metadata = json.loads(
    subprocess.check_output(
        ["cargo", "metadata", "--no-deps", "--format-version", "1"],
        text=True,
    )
)
workspace_members = set(metadata["workspace_members"])
packages = [
    package["name"]
    for package in metadata["packages"]
    if package["id"] in workspace_members
]

args = ["cargo", "fmt"]
for package in packages:
    args.extend(["--package", package])
args.extend(["--", "--check"])
subprocess.run(args, check=True)
