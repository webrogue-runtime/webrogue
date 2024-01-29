#pragma once

#include "binary-writer.h"
#include "ir.h"
#include "stream.h"

namespace wabt {
inline Result write_file(const WASMModule *file, Stream *out_stream) {
    WriteBinaryOptions write_options;
    CHECK_RESULT(WriteBinaryModule(out_stream, file, write_options));
    return Result::Ok;
}
} // namespace wabt