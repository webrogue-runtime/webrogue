import sys
import os

prefix_str_1 = "lld-link: Loaded webrogue_aot_lib.lib("
prefix_str_2 = "lld-link: Reading webrogue_aot_lib.lib("

visited = set()

for win_type in ["gui", "console"]:
    for line in open(f"lld_output_{win_type}.txt", "r").readlines():
        if line.startswith(prefix_str_1):
            line = line[len(prefix_str_1) : line.find(")", len(prefix_str_1))]
            visited.add(line)
        elif line.startswith(prefix_str_2):
            line = line[len(prefix_str_2) : -2]
            visited.add(line)

for line in open("lib_content.txt", "r").readlines():
    line = line.strip()
    if line.endswith(".dll"):
        continue
    if [1 for o in visited if line.endswith(o)]:
        continue
    print(line)
