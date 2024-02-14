import lzma
import os
import sys
import pathlib
import os
import zstd

if sys.argv[1] == "normal":
    pack_wasm = True
elif sys.argv[1] == "nowasm":
    pack_wasm = False
else:
    print(f"Invalid mode: {sys.argv[1]}", file=sys.stderr)
    exit(1)

print(f"packing args: {sys.argv}")


# first arg is output path
output_path = sys.argv[2]

# second arg is mod directory path
mod_dir_path = pathlib.Path(sys.argv[3])

mod_name = mod_dir_path.parts[-1]

is_core = mod_name == "core"

if not mod_dir_path.joinpath("mod.a").exists():
    print(f"{mod_dir_path.joinpath('mod.a')} does not exists", file=sys.stderr)
    exit(1)
if is_core and not mod_dir_path.joinpath("stdlibs.a").exists():
    print(f"{mod_dir_path.joinpath('mod.a')} does not exists", file=sys.stderr)
    exit(1)

data_to_compress = b""

output_file = open(
    output_path,
    mode="wb",
)


def encode_num(num: int) -> bytes:
    result = hex(num)[2:]
    result = "0" * (16 - len(result)) + result
    return result.encode()


def compress_file(path: str, is_wasm: bool):
    global data_to_compress
    absolute_path = mod_dir_path.joinpath(path)
    if is_wasm and not pack_wasm:
        data = b""
    else:
        data = open(absolute_path, "rb").read()
    print(f"compress_file path: {path} size: {len(data)}")
    data_to_compress += path.encode() + b"\0" + encode_num(len(data)) + b"\0" + data


compress_file("mod.a", is_wasm=True)
if is_core:
    compress_file("stdlibs.a", is_wasm=True)
res_path = mod_dir_path.joinpath("wrres")
if os.path.isdir(res_path):
    for dir, _, files in os.walk(res_path):
        relPath = os.path.relpath(dir, mod_dir_path)
        for file in files:
            compress_file(os.path.join(relPath, file), is_wasm=False)

# raw, zstd, xz
compressor_name = "zstd"
if len(data_to_compress) < 1024 * 1024:
    compressor_name = "xz"

if compressor_name == "zstd":
    compressed = zstd.compress(
        data_to_compress,
        1,
    )
elif compressor_name == "xz":
    compressed = lzma.compress(
        data_to_compress,
        format=lzma.FORMAT_XZ,
        check=lzma.CHECK_CRC32,
    )
elif compressor_name == "raw":
    compressed = data_to_compress

output_file.write(mod_name.encode() + b"\0")
output_file.write(compressor_name.encode() + b"\0")
output_file.write(encode_num(len(data_to_compress)) + b"\0")
output_file.write(compressed)
output_file.close()
