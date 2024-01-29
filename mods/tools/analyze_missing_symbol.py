import pathlib
import subprocess

mod_names: list[str] = ["core", "broguece_webrogue"]

mods_dir: str = str(pathlib.Path(__file__).parent.parent)

linker_path: str = (
    open(mods_dir + "/tools/linker_path.cmake")
    .read()
    .strip()
    .replace("set(WEBROGUE_MODS_LINKER ", "")
    .replace(")", "")
)

ret: int = subprocess.call(
    [
        linker_path,
        "--export=wr_start",
        "--export=__wasm_call_ctors",
        "--no-entry",
    ]
    + [mods_dir + "/" + mod_name + "/mod.a" for mod_name in mod_names]
    + ["--export=init_mod_" + mod_name for mod_name in mod_names]
    + [
        "-o",
        mods_dir + "/tools/import_analyze.wasm",
        "--fatal-warnings",
        "--allow-undefined",
    ]
)
if ret:
    print(f"wasm-ld returned {ret}")
    exit(ret)

ret = subprocess.call(
    [
        "wasm2wat",
        mods_dir + "/tools/import_analyze.wasm",
        "-o",
        mods_dir + "/tools/import_analyze.wat",
    ]
)
if ret:
    print(f"wasm2wat returned {ret}")
    exit(ret)

missing_imports: list[str] = []

call_map: dict[str, list[str]] = dict()

current_func_name: str = ""

for line in open(mods_dir + "/tools/import_analyze.wat").readlines():
    line: str = line.strip()
    if line.startswith("(import") and not line.startswith('(import "webrogue"'):
        func_name_start_pos = line.find("func $") + 6
        func_name_end_pos = line.find(" ", func_name_start_pos)
        missing_imports.append(line[func_name_start_pos:func_name_end_pos])
    if line.startswith("(func $"):
        func_name_start_pos = 7
        func_name_end_pos = line.find(" (type ", func_name_start_pos)
        current_func_name = line[func_name_start_pos:func_name_end_pos]
        call_map[current_func_name] = []
    if line.startswith("call $"):
        func_name_start_pos = 6
        func_name: str = line[func_name_start_pos:]
        if func_name.endswith(")"):
            func_name = func_name[:-1]
        call_map[current_func_name].append(func_name)

inverted_call_map: dict[str, list[str]] = dict()

for caller_func, called_funcs in call_map.items():
    for called_func in called_funcs:
        if called_func not in inverted_call_map.keys():
            inverted_call_map[called_func] = []
        inverted_call_map[called_func].append(caller_func)

forbidden_func_names: list[str] = [""]

visited_funcs: set[str] = set()


with open(mods_dir + "/tools/import_analyze.dot", "w") as file:
    print("digraph mygraph {", file=file)
    print("    overlap=prism", file=file)
    print("    overlap_scaling=-10", file=file)
    print('    fontname="Helvetica,Arial,sans-serif"', file=file)
    print('    node [fontname="Helvetica,Arial,sans-serif"]', file=file)
    print('    edge [fontname="Helvetica,Arial,sans-serif"]', file=file)
    print("    node [shape=box];", file=file)
    print("", file=file)

    def visit_func(func_name: str):
        visited_funcs.add(func_name)
        for called_func_name in inverted_call_map[func_name]:
            print('    "' + called_func_name + '" -> "' + func_name + '"', file=file)
            if called_func_name not in visited_funcs:
                if called_func_name in inverted_call_map.keys():
                    visit_func(called_func_name)

    for func_name in missing_imports:
        print('    "' + func_name + '" [color="red"];', file=file)
        visit_func(func_name)

    for forbidden_func_name in forbidden_func_names:
        if forbidden_func_name in inverted_call_map.keys():
            print('    "' + forbidden_func_name + '" [color="red"];', file=file)
            visit_func(forbidden_func_name)

    print("}\n", file=file)
    file.close()

for layout_engine in ["fdp", "twopi", "neato", "dot"]:
    print(layout_engine)
    ret = subprocess.call(
        [
            layout_engine,
            mods_dir + "/tools/import_analyze.dot",
            "-Tpng",
            "-o",
            mods_dir + "/tools/import_analyze_" + layout_engine + ".png",
        ]
    )
    if ret:
        print(f"{layout_engine} returned {ret}")
        exit(ret)
