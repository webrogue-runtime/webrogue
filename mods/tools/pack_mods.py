import lzma
import sys
import pathlib


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

if  not mod_dir_path.joinpath("mod.a").exists():
    print(f"{mod_dir_path.joinpath('mod.a')} does not exists", file=sys.stderr)
    exit(1)
if is_core and not mod_dir_path.joinpath("stdlibs.a").exists():
    print(f"{mod_dir_path.joinpath('mod.a')} does not exists", file=sys.stderr)
    exit(1)

compressor = lzma.LZMACompressor(format=lzma.FORMAT_XZ, check=lzma.CHECK_CRC32)
compressed = b""

output_file = open(
    output_path,
    mode="wb",
)

decompressed_size = 0


def append_data(data: bytes):
    global compressed
    global decompressed_size
    compressed += compressor.compress(data)
    decompressed_size += len(data)


def encode_num(num: int) -> bytes:
    result = hex(num)[2:]
    result = "0" * (16 - len(result)) + result
    return result.encode()


def compress_file(path: str, is_wasm: bool):
    global compressed
    absolute_path = mod_dir_path.joinpath(path)
    if is_wasm and not pack_wasm:
        data = b""
    else:
        data = open(absolute_path, "rb").read()
    print(f"compress_file path: {path} size: {len(data)}")
    append_data(path.encode() + b"\0" + encode_num(len(data)) + b"\0" + data)

compress_file("mod.a", is_wasm=True)
if is_core:
    compress_file("stdlibs.a", is_wasm=True)

compressed += compressor.flush()

output_file.write(mod_name.encode() + b"\0")
output_file.write(encode_num(decompressed_size) + b"\0")
output_file.write(compressed)
output_file.close()
