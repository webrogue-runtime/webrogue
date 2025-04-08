import os
import subprocess
import requests
import zipfile

template_dir = os.path.dirname(os.path.realpath(__file__))
repo_dir = os.path.dirname(os.path.dirname(template_dir))

sdk_dir = os.environ["WindowsSdkDir"]
sdk_version = os.environ["WindowsSDKVersion"].removesuffix("\\")
vc_tools_install_dir = os.environ["VCToolsInstallDir"]

subprocess.run(
    [
        "cargo",
        "build",
        "--manifest-path=../../crates/aot-lib/Cargo.toml",
        "--target-dir=./target",
        "--target=x86_64-pc-windows-msvc",
        "--features=gfx-fallback-cmake",
        "--profile",
        "release-lto",
    ],
    cwd=str(template_dir),
).check_returncode()

out_dir = os.path.join(repo_dir, "aot_artifacts", "x86_64-windows-msvc")
if not os.path.exists(out_dir):
    os.makedirs(out_dir)

for win_type in ["gui", "console"]:
    subprocess.run(
        [
            "clang",
            "-target",
            "x86_64-pc-win32",
            "-c",
            "main.c",
            "-o",
            f"{win_type}.obj",
            "-fms-compatibility-version=19",
            "-fms-extensions",
            "-fdelayed-template-parsing",
            "-fexceptions",
            "-mthread-model",
            "posix",
            "-fno-threadsafe-statics",
            "-Wno-msvc-not-found",
            "-DWIN32",
            "-D_WIN32",
            "-D_MT",
            "-D_DLL",
            "-Xclang",
            "-disable-llvm-verifier",
            "-D_CRT_SECURE_NO_WARNINGS",
            "-D_CRT_NONSTDC_NO_DEPRECATE",
            "-U__GNUC__",
            "-U__gnu_linux__",
            "-U__GNUC_MINOR__",
            "-U__GNUC_PATCHLEVEL__",
            "-U__GNUC_STDC_INLINE__",
            f"-I{os.path.join(sdk_dir, 'Include', sdk_version , 'um')}",
            f"-I{os.path.join(sdk_dir, 'Include', sdk_version , 'shared')}",
            # "-I$XWIN_PATH/crt/include",
            f"-I{os.path.join(sdk_dir, 'Include', sdk_version , 'ucrt')}",
            f"-DWR_WIN_TYPE_{win_type}",
        ],
        cwd=str(template_dir),
    ).check_returncode()
    obj_out_path = os.path.join(out_dir, f"{win_type}.obj")
    if os.path.exists(obj_out_path):
        os.remove(obj_out_path)
    os.rename(
        os.path.join(template_dir, f"{win_type}.obj"),
        obj_out_path,
    )
um_lib_dir = os.path.join(sdk_dir, "Lib", sdk_version, "um", "x64")

webrogue_aot_lib_path = os.path.join(template_dir, "webrogue_aot_lib.lib")
if os.path.exists(webrogue_aot_lib_path):
    os.remove(webrogue_aot_lib_path)
os.rename(
    os.path.join(
        template_dir,
        "target",
        "x86_64-pc-windows-msvc",
        "release-lto",
        "webrogue_aot_lib.lib",
    ),
    webrogue_aot_lib_path,
)

subprocess.run(
    [
        "llvm-ar",
        "qLs",
        webrogue_aot_lib_path,
        os.path.join(vc_tools_install_dir, "lib", "x64", "libcpmt.lib"),
        os.path.join(vc_tools_install_dir, "lib", "x64", "libvcruntime.lib"),
        os.path.join(vc_tools_install_dir, "lib", "x64", "oldnames.lib"),
        os.path.join(vc_tools_install_dir, "lib", "x64", "libcmt.lib"),
        os.path.join(sdk_dir, "Lib", sdk_version, "ucrt", "x64", "libucrt.lib"),
        os.path.join(um_lib_dir, "ws2_32.lib"),
        os.path.join(um_lib_dir, "ntdll.lib"),
        os.path.join(um_lib_dir, "AdvAPI32.Lib"),
        os.path.join(um_lib_dir, "bcrypt.lib"),
        os.path.join(um_lib_dir, "kernel32.lib"),
        os.path.join(um_lib_dir, "UserEnv.Lib"),
        os.path.join(um_lib_dir, "oleaut32.lib"),
        os.path.join(um_lib_dir, "ole32.lib"),
        os.path.join(um_lib_dir, "gdi32.lib"),
        os.path.join(um_lib_dir, "user32.lib"),
        os.path.join(um_lib_dir, "imm32.lib"),
        os.path.join(um_lib_dir, "version.lib"),
        os.path.join(um_lib_dir, "setupapi.lib"),
        os.path.join(um_lib_dir, "winmm.lib"),
        os.path.join(um_lib_dir, "shell32.lib"),
        os.path.join(um_lib_dir, "uuid.lib"),
    ],
    cwd=str(template_dir),
).check_returncode()

lib_content_result = subprocess.run(
    ["llvm-ar", "t", webrogue_aot_lib_path],
    cwd=str(template_dir),
    stdout=subprocess.PIPE,
)
if lib_content_result.returncode:
    print(lib_content_result.stdout.decode())
    quit()
lib_content = lib_content_result.stdout.decode()

lld_outputs: list[str] = []

for win_type in ["gui", "console"]:
    obj_out_path = os.path.join(out_dir, f"{win_type}.obj")
    exe_path = os.path.join(template_dir, "aot.exe")
    # Collect verbose information to preform tree-shaking of resulting static archives
    lld_content_result = subprocess.run(
        [
            "lld-link",
            f"-out:{exe_path}",
            "-nologo",
            "-machine:x64",
            os.path.join(template_dir, "empty.obj"),
            obj_out_path,
            webrogue_aot_lib_path,
            "/nodefaultlib",
            "/threads:1",
            "/verbose",
        ],
        stderr=subprocess.PIPE,
        cwd=str(template_dir),
    )
    if lld_content_result.returncode:
        print(lld_content_result.stderr.decode())
        quit()
    lld_outputs.append(lld_content_result.stderr.decode())

prefix_str_1 = "lld-link: Loaded webrogue_aot_lib.lib("
prefix_str_2 = "lld-link: Reading webrogue_aot_lib.lib("
visited = set()

for lld_output in lld_outputs:
    for line in lld_output.splitlines():
        if line.startswith(prefix_str_1):
            line = line[len(prefix_str_1) : line.find(")", len(prefix_str_1))]
            visited.add(line)
        elif line.startswith(prefix_str_2):
            line = line[len(prefix_str_2) : -2]
            visited.add(line)

objects_to_remove = []

for line in lib_content.splitlines():
    line = line.strip()
    if line.endswith(".dll"):
        continue
    if [1 for o in visited if line.endswith(o)]:
        continue
    objects_to_remove.append(line)


def batched(iterable, n):
    l = len(iterable)
    for ndx in range(0, l, n):
        yield iterable[ndx : min(ndx + n, l)]


for object_batch_to_remove in batched(objects_to_remove, 128):
    subprocess.run(
        ["llvm-ar", "d", webrogue_aot_lib_path] + list(object_batch_to_remove),
        cwd=str(template_dir),
    ).check_returncode()

webrogue_aot_lib_out_path = os.path.join(out_dir, "webrogue_aot_lib.lib")
if os.path.exists(webrogue_aot_lib_out_path):
    os.remove(webrogue_aot_lib_out_path)
os.rename(
    webrogue_aot_lib_path,
    webrogue_aot_lib_out_path,
)

angle_zip_path = os.path.join(os.path.dirname(template_dir), "angle_windows_x64.zip")
if not os.path.exists(angle_zip_path):
    response = requests.get(
        "https://github.com/webrogue-runtime/angle-builder/releases/latest/download/windows_x64.zip",
        allow_redirects=True,
    )
    open(angle_zip_path, "wb").write(response.content)

zip = zipfile.ZipFile(angle_zip_path)
for libname in ["libEGL.dll", "libGLESv2.dll"]:
    lib_path = os.path.join(os.path.dirname(template_dir), libname)
    if not os.path.exists(lib_path):
        open(lib_path, "wb").write(zip.read(f"x64/{libname}"))
    lib_out_path = os.path.join(out_dir, libname)
    if os.path.exists(lib_out_path):
        os.remove(lib_out_path)
    os.rename(
        lib_path,
        lib_out_path,
    )
