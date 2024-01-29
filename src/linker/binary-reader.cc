/*
 * Copyright 2016 WebAssembly Community Group participants
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

#include "binary-reader.h"

#include <cassert>
#include <cstdint>
#include <cstdio>
#include <cstring>
#include <stack>
#include <vector>

#include "config.h"

#include "binary.h"
#include "leb128.h"
#include "utf8.h"

#if HAVE_ALLOCA
#include <alloca.h>
#endif

#define ERROR_IF(expr, ...)                                                    \
    do {                                                                       \
        if (expr) {                                                            \
            PrintError(__VA_ARGS__);                                           \
            return Result::Error;                                              \
        }                                                                      \
    } while (0)

#define ERROR_UNLESS(expr, ...) ERROR_IF(!(expr), __VA_ARGS__)

#define ERROR_UNLESS_OPCODE_ENABLED(opcode)                                    \
    do {                                                                       \
        if (!opcode.IsEnabled(options_.features)) {                            \
            return ReportUnexpectedOpcode(opcode);                             \
        }                                                                      \
    } while (0)

#define CALLBACK0(member)                                                      \
    ERROR_UNLESS(succeeded(delegate->member()), #member " callback failed")

#define CALLBACK(member, ...)                                                  \
    ERROR_UNLESS(succeeded(delegate->member(__VA_ARGS__)),                     \
                 #member " callback failed")

namespace wabt {

namespace {

class BinaryReader {
public:
    struct ReadModuleOptions {
        bool stopOnFirstError;
    };

    BinaryReader(const void *data, size_t size, BinaryReaderDelegate *delegate,
                 const ReadBinaryOptions &options);

    Result readModule(const ReadModuleOptions &options);

private:
    template <typename T, T BinaryReader::*member> struct ValueRestoreGuard {
        explicit ValueRestoreGuard(BinaryReader *reader)
            : reader(reader), previousValue(reader->*member) {
        }
        ~ValueRestoreGuard() {
            reader->*member = previousValue;
        }

        BinaryReader *reader;
        T previousValue;
    };

    struct ReadSectionsOptions {
        bool stopOnFirstError;
    };

    void WABT_PRINTF_FORMAT(2, 3) PrintError(const char *format, ...);
    [[nodiscard]] Result readOpcode(Opcode *outValue, const char *desc);
    template <typename T>
    [[nodiscard]] Result readT(T *outValue, const char *typeName,
                               const char *desc);
    [[nodiscard]] Result readU8(uint8_t *outValue, const char *desc);
    [[nodiscard]] Result readU32(uint32_t *outValue, const char *desc);
    [[nodiscard]] Result readF32(uint32_t *outValue, const char *desc);
    [[nodiscard]] Result readF64(uint64_t *outValue, const char *desc);
    [[nodiscard]] Result readV128(v128 *outValue, const char *desc);
    [[nodiscard]] Result readU32Leb128(uint32_t *outValue, const char *desc);
    [[nodiscard]] Result readU64Leb128(uint64_t *outValue, const char *desc);
    [[nodiscard]] Result readS32Leb128(uint32_t *outValue, const char *desc);
    [[nodiscard]] Result readS64Leb128(uint64_t *outValue, const char *desc);
    [[nodiscard]] Result readType(Type *outValue, const char *desc);
    [[nodiscard]] Result readRefType(Type *outValue, const char *desc);
    [[nodiscard]] Result readExternalKind(ExternalKind *outValue,
                                          const char *desc);
    [[nodiscard]] Result readStr(StringRef *outStr, const char *desc);
    [[nodiscard]] Result readBytes(const void **outData, Address *outDataSize,
                                   const char *desc);
    [[nodiscard]] Result readBytesWithSize(const void **outData, Offset size,
                                           const char *desc);
    [[nodiscard]] Result readIndex(Index *index, const char *desc);
    [[nodiscard]] Result readOffset(Offset *offset, const char *desc);
    [[nodiscard]] Result readAlignment(Address *alignLog2, const char *desc);
    [[nodiscard]] Result readMemidx(Index *memidx, const char *desc);
    [[nodiscard]] Result readMemLocation(Address *alignmentLog2, Index *memidx,
                                         Address *offset, const char *descAlign,
                                         const char *descMemidx,
                                         const char *descOffset,
                                         uint8_t *laneVal = nullptr);
    [[nodiscard]] Result callbackMemLocation(const Address *alignmentLog2,
                                             const Index *memidx,
                                             const Address *offset,
                                             const uint8_t *laneVal = nullptr);
    [[nodiscard]] Result readCount(Index *index, const char *desc);
    [[nodiscard]] Result readField(TypeMut *outValue);

    bool isConcreteType(Type);
    bool isBlockType(Type);

    Index numTotalFuncs();

    [[nodiscard]] Result readInitExpr(Index index);
    [[nodiscard]] Result readTable(Type *outElemType, Limits *outElemLimits);
    [[nodiscard]] Result readMemory(Limits *outPageLimits);
    [[nodiscard]] Result readGlobalHeader(Type *outType, bool *outMutable);
    [[nodiscard]] Result readTagType(Index *outSigIndex);
    [[nodiscard]] Result readAddress(Address *outValue, Index memory,
                                     const char *desc);
    [[nodiscard]] Result readFunctionBody(Offset endOffset);
    // ReadInstructions reads until end_offset or the nesting depth reaches
    // zero.
    [[nodiscard]] Result readInstructions(Offset endOffset,
                                          const char *context);
    [[nodiscard]] Result readNameSection(Offset sectionSize);
    [[nodiscard]] Result readRelocSection(Offset sectionSize);
    [[nodiscard]] Result readDylinkSection(Offset sectionSize);
    [[nodiscard]] Result readGenericCustomSection(StringRef name,
                                                  Offset sectionSize);
    [[nodiscard]] Result readDylink0Section(Offset sectionSize);
    [[nodiscard]] Result readTargetFeaturesSections(Offset sectionSize);
    [[nodiscard]] Result readLinkingSection(Offset sectionSize);
    [[nodiscard]] Result readCodeMetadataSection(StringRef name,
                                                 Offset sectionSize);
    [[nodiscard]] Result readCustomSection(Index sectionIndex,
                                           Offset sectionSize);
    [[nodiscard]] Result readTypeSection(Offset sectionSize);
    [[nodiscard]] Result readImportSection(Offset sectionSize);
    [[nodiscard]] Result readFunctionSection(Offset sectionSize);
    [[nodiscard]] Result readTableSection(Offset sectionSize);
    [[nodiscard]] Result readMemorySection(Offset sectionSize);
    [[nodiscard]] Result readGlobalSection(Offset sectionSize);
    [[nodiscard]] Result readExportSection(Offset sectionSize);
    [[nodiscard]] Result readStartSection(Offset sectionSize);
    [[nodiscard]] Result readElemSection(Offset sectionSize);
    [[nodiscard]] Result readCodeSection(Offset sectionSize);
    [[nodiscard]] Result readDataSection(Offset sectionSize);
    [[nodiscard]] Result readDataCountSection(Offset sectionSize);
    [[nodiscard]] Result readTagSection(Offset sectionSize);
    [[nodiscard]] Result readSections(const ReadSectionsOptions &options);
    Result ReportUnexpectedOpcode(Opcode opcode, const char *message = nullptr);

    size_t readEnd = 0; // Either the section end or data_size.
    BinaryReaderDelegate::State state;
    BinaryReaderDelegate *delegate = nullptr;
    TypeVector paramTypes;
    TypeVector resultTypes;
    TypeMutVector fields;
    std::vector<Index> targetDepths;
    const ReadBinaryOptions &options_;
    BinarySection lastKnownSection = BinarySection::Invalid;
    bool didReadNamesSection = false;
    bool readingCustomSection = false;
    Index numFuncImports = 0;
    Index numTableImports = 0;
    Index numMemoryImports = 0;
    Index numGlobalImports = 0;
    Index numTagImports = 0;
    Index numFunctionSignatures = 0;
    Index numFunctionBodies = 0;
    Index dataCount = kInvalidIndex;

    using ReadEndRestoreGuard =
        ValueRestoreGuard<size_t, &BinaryReader::readEnd>;
};

BinaryReader::BinaryReader(const void *data, size_t size,
                           BinaryReaderDelegate *delegate,
                           const ReadBinaryOptions &options)
    : readEnd(size), state(static_cast<const uint8_t *>(data), size),
      delegate(delegate), options_(options),
      lastKnownSection(BinarySection::Invalid) {
    delegate->OnSetState(&state);
}

void WABT_PRINTF_FORMAT(2, 3) BinaryReader::PrintError(const char *format,
                                                       ...) {
}

Result BinaryReader::ReportUnexpectedOpcode(Opcode opcode, const char *where) {
    std::string message = "unexpected opcode";
    if (where) {
        message += ' ';
        message += where;
    }

    message += ":";

    std::vector<uint8_t> bytes = opcode.GetBytes();
    assert(bytes.size() > 0);

    for (uint8_t byte : bytes) {
        message += StringPrintf(" 0x%x", byte);
    }

    PrintError("%s", message.c_str());
    return Result::Error;
}

Result BinaryReader::readOpcode(Opcode *outValue, const char *desc) {
    uint8_t value = 0;
    CHECK_RESULT(readU8(&value, desc));

    if (Opcode::IsPrefixByte(value)) {
        uint32_t code;
        CHECK_RESULT(readU32Leb128(&code, desc));
        *outValue = Opcode::FromCode(value, code);
    } else {
        *outValue = Opcode::FromCode(value);
    }
    return Result::Ok;
}

template <typename T>
Result BinaryReader::readT(T *outValue, const char *typeName,
                           const char *desc) {
    if (state.offset + sizeof(T) > readEnd) {
        PrintError("unable to read %s: %s", typeName, desc);
        return Result::Error;
    }
#if WABT_BIG_ENDIAN
    uint8_t tmp[sizeof(T)];
    memcpy(tmp, state_.data + state_.offset, sizeof(tmp));
    SwapBytesSized(tmp, sizeof(tmp));
    memcpy(out_value, tmp, sizeof(T));
#else
    memcpy(outValue, state.data + state.offset, sizeof(T));
#endif
    state.offset += sizeof(T);
    return Result::Ok;
}

Result BinaryReader::readU8(uint8_t *outValue, const char *desc) {
    return readT(outValue, "uint8_t", desc);
}

Result BinaryReader::readU32(uint32_t *outValue, const char *desc) {
    return readT(outValue, "uint32_t", desc);
}

Result BinaryReader::readF32(uint32_t *outValue, const char *desc) {
    return readT(outValue, "float", desc);
}

Result BinaryReader::readF64(uint64_t *outValue, const char *desc) {
    return readT(outValue, "double", desc);
}

Result BinaryReader::readV128(v128 *outValue, const char *desc) {
    return readT(outValue, "v128", desc);
}

Result BinaryReader::readU32Leb128(uint32_t *outValue, const char *desc) {
    const uint8_t *p = state.data + state.offset;
    const uint8_t *end = state.data + readEnd;
    size_t bytesRead = wabt::ReadU32Leb128(p, end, outValue);
    ERROR_UNLESS(bytesRead > 0, "unable to read u32 leb128: %s", desc);
    state.offset += bytesRead;
    return Result::Ok;
}

Result BinaryReader::readU64Leb128(uint64_t *outValue, const char *desc) {
    const uint8_t *p = state.data + state.offset;
    const uint8_t *end = state.data + readEnd;
    size_t bytesRead = wabt::ReadU64Leb128(p, end, outValue);
    ERROR_UNLESS(bytesRead > 0, "unable to read u64 leb128: %s", desc);
    state.offset += bytesRead;
    return Result::Ok;
}

Result BinaryReader::readS32Leb128(uint32_t *outValue, const char *desc) {
    const uint8_t *p = state.data + state.offset;
    const uint8_t *end = state.data + readEnd;
    size_t bytesRead = wabt::ReadS32Leb128(p, end, outValue);
    ERROR_UNLESS(bytesRead > 0, "unable to read i32 leb128: %s", desc);
    state.offset += bytesRead;
    return Result::Ok;
}

Result BinaryReader::readS64Leb128(uint64_t *outValue, const char *desc) {
    const uint8_t *p = state.data + state.offset;
    const uint8_t *end = state.data + readEnd;
    size_t bytesRead = wabt::ReadS64Leb128(p, end, outValue);
    ERROR_UNLESS(bytesRead > 0, "unable to read i64 leb128: %s", desc);
    state.offset += bytesRead;
    return Result::Ok;
}

Result BinaryReader::readType(Type *outValue, const char *desc) {
    uint32_t type = 0;
    CHECK_RESULT(readS32Leb128(&type, desc));
    if (static_cast<Type::Enum>(type) == Type::Reference) {
        uint32_t heapType = 0;
        CHECK_RESULT(readS32Leb128(&heapType, desc));
        *outValue = Type(Type::Reference, heapType);
    } else {
        *outValue = static_cast<Type>(type);
    }
    return Result::Ok;
}

Result BinaryReader::readRefType(Type *outValue, const char *desc) {
    uint32_t type = 0;
    CHECK_RESULT(readS32Leb128(&type, desc));
    *outValue = static_cast<Type>(type);
    ERROR_UNLESS(outValue->IsRef(), "%s must be a reference type", desc);
    return Result::Ok;
}

Result BinaryReader::readExternalKind(ExternalKind *outValue,
                                      const char *desc) {
    uint8_t value = 0;
    CHECK_RESULT(readU8(&value, desc));
    ERROR_UNLESS(value < kExternalKindCount, "invalid export external kind: %d",
                 value);
    *outValue = static_cast<ExternalKind>(value);
    return Result::Ok;
}

Result BinaryReader::readStr(StringRef *outStr, const char *desc) {
    uint32_t strLen = 0;
    CHECK_RESULT(readU32Leb128(&strLen, "string length"));

    ERROR_UNLESS(state.offset + strLen <= readEnd, "unable to read string: %s",
                 desc);

    *outStr = StringRef(
        reinterpret_cast<const char *>(state.data) + state.offset, strLen);
    state.offset += strLen;

    ERROR_UNLESS(IsValidUtf8(outStr->data(), outStr->size()),
                 "invalid utf-8 encoding: %s", desc);
    return Result::Ok;
}

Result BinaryReader::readBytes(const void **outData, Address *outDataSize,
                               const char *desc) {
    uint32_t dataSize = 0;
    CHECK_RESULT(readU32Leb128(&dataSize, "data size"));
    CHECK_RESULT(readBytesWithSize(outData, dataSize, desc));
    *outDataSize = dataSize;
    return Result::Ok;
}

Result BinaryReader::readBytesWithSize(const void **outData, Offset size,
                                       const char *desc) {
    ERROR_UNLESS(state.offset + size <= readEnd, "unable to read data: %s",
                 desc);

    *outData = static_cast<const uint8_t *>(state.data) + state.offset;
    state.offset += size;
    return Result::Ok;
}

Result BinaryReader::readIndex(Index *index, const char *desc) {
    uint32_t value;
    CHECK_RESULT(readU32Leb128(&value, desc));
    *index = value;
    return Result::Ok;
}

Result BinaryReader::readOffset(Offset *offset, const char *desc) {
    uint32_t value;
    CHECK_RESULT(readU32Leb128(&value, desc));
    *offset = value;
    return Result::Ok;
}

Result BinaryReader::readAlignment(Address *alignmentLog2, const char *desc) {
    uint32_t value;
    CHECK_RESULT(readU32Leb128(&value, desc));
    if (value >= 128 ||
        (value >= 32 && !options_.features.multi_memory_enabled())) {
        PrintError("invalid %s: %u", desc, value);
        return Result::Error;
    }
    *alignmentLog2 = value;
    return Result::Ok;
}

Result BinaryReader::readMemidx(Index *memidx, const char *desc) {
    CHECK_RESULT(readIndex(memidx, desc));
    return Result::Ok;
}

Result BinaryReader::readMemLocation(Address *alignmentLog2, Index *memidx,
                                     Address *offset, const char *descAlign,
                                     const char *descMemidx,
                                     const char *descOffset, uint8_t *laneVal) {
    CHECK_RESULT(readAlignment(alignmentLog2, descAlign));
    *memidx = 0;
    if (*alignmentLog2 >> 6) {
        ERROR_IF(!options_.features.multi_memory_enabled(),
                 "multi_memory not allowed");
        *alignmentLog2 = *alignmentLog2 & ((1 << 6) - 1);
        CHECK_RESULT(readMemidx(memidx, descMemidx));
    }
    CHECK_RESULT(readAddress(offset, 0, descOffset));

    if (laneVal) {
        CHECK_RESULT(readU8(laneVal, "Lane idx"));
    }

    return Result::Ok;
}

Result BinaryReader::callbackMemLocation(const Address *alignmentLog2,
                                         const Index *memidx,
                                         const Address *offset,
                                         const uint8_t *laneVal) {
    if (laneVal) {
        if (*memidx) {
            CALLBACK(OnOpcodeUint32Uint32Uint32Uint32, *alignmentLog2, *memidx,
                     *offset, *laneVal);
        } else {
            CALLBACK(OnOpcodeUint32Uint32Uint32, *alignmentLog2, *offset,
                     *laneVal);
        }
    } else {
        if (*memidx) {
            CALLBACK(OnOpcodeUint32Uint32Uint32, *alignmentLog2, *memidx,
                     *offset);
        } else {
            CALLBACK(OnOpcodeUint32Uint32, *alignmentLog2, *offset);
        }
    }

    return Result::Ok;
}

Result BinaryReader::readCount(Index *count, const char *desc) {
    CHECK_RESULT(readIndex(count, desc));

    // This check assumes that each item follows in this section, and takes at
    // least 1 byte. It's possible that this check passes but reading fails
    // later. It is still useful to check here, though, because it early-outs
    // when an erroneous large count is used, before allocating memory for it.
    size_t sectionRemaining = readEnd - state.offset;
    if (*count > sectionRemaining) {
        return Result::Error;
    }
    return Result::Ok;
}

Result BinaryReader::readField(TypeMut *outValue) {
    // TODO: Reuse for global header too?
    Type fieldType;
    CHECK_RESULT(readType(&fieldType, "field type"));
    ERROR_UNLESS(isConcreteType(fieldType),
                 "expected valid field type (got " PRItypecode ")",
                 WABT_PRINTF_TYPE_CODE(fieldType));

    uint8_t isMutable = 0;
    CHECK_RESULT(readU8(&isMutable, "field mutability"));
    ERROR_UNLESS(isMutable <= 1, "field mutability must be 0 or 1");
    outValue->type = fieldType;
    outValue->isMutable = isMutable;
    return Result::Ok;
}

bool BinaryReader::isConcreteType(Type type) {
    switch (type) {
    case Type::I32:
    case Type::I64:
    case Type::F32:
    case Type::F64:
        return true;

    case Type::V128:
        return options_.features.simd_enabled();

    case Type::FuncRef:
    case Type::ExternRef:
        return options_.features.reference_types_enabled();

    case Type::Reference:
        return options_.features.function_references_enabled();

    default:
        return false;
    }
}

bool BinaryReader::isBlockType(Type type) {
    if (isConcreteType(type) || type == Type::Void) {
        return true;
    }

    if (!(options_.features.multi_value_enabled() && type.IsIndex())) {
        return false;
    }

    return true;
}

Index BinaryReader::numTotalFuncs() {
    return numFuncImports + numFunctionSignatures;
}

Result BinaryReader::readInitExpr(Index index) {
    CHECK_RESULT(readInstructions(readEnd, "init expression"));
    assert(state.offset <= readEnd);
    return Result::Ok;
}

Result BinaryReader::readTable(Type *outElemType, Limits *outElemLimits) {
    CHECK_RESULT(readRefType(outElemType, "table elem type"));

    uint8_t flags;
    uint32_t initial;
    uint32_t max = 0;
    CHECK_RESULT(readU8(&flags, "table flags"));
    bool hasMax = flags & WABT_BINARY_LIMITS_HAS_MAX_FLAG;
    bool isShared = flags & WABT_BINARY_LIMITS_IS_SHARED_FLAG;
    bool is64 = flags & WABT_BINARY_LIMITS_IS_64_FLAG;
    const uint8_t unknownFlags = flags & ~WABT_BINARY_LIMITS_ALL_FLAGS;
    ERROR_IF(isShared, "tables may not be shared");
    ERROR_IF(is64, "tables may not be 64-bit");
    ERROR_UNLESS(unknownFlags == 0, "malformed table limits flag: %d", flags);
    CHECK_RESULT(readU32Leb128(&initial, "table initial elem count"));
    if (hasMax) {
        CHECK_RESULT(readU32Leb128(&max, "table max elem count"));
    }

    outElemLimits->has_max = hasMax;
    outElemLimits->initial = initial;
    outElemLimits->max = max;
    return Result::Ok;
}

Result BinaryReader::readMemory(Limits *outPageLimits) {
    uint8_t flags;
    uint64_t initial;
    uint64_t max = 0;
    CHECK_RESULT(readU8(&flags, "memory flags"));
    bool hasMax = flags & WABT_BINARY_LIMITS_HAS_MAX_FLAG;
    bool isShared = flags & WABT_BINARY_LIMITS_IS_SHARED_FLAG;
    bool is64 = flags & WABT_BINARY_LIMITS_IS_64_FLAG;
    const uint8_t unknownFlags = flags & ~WABT_BINARY_LIMITS_ALL_FLAGS;
    ERROR_UNLESS(unknownFlags == 0, "malformed memory limits flag: %d", flags);
    ERROR_IF(isShared && !options_.features.threads_enabled(),
             "memory may not be shared: threads not allowed");
    ERROR_IF(is64 && !options_.features.memory64_enabled(),
             "memory64 not allowed");
    if (options_.features.memory64_enabled()) {
        CHECK_RESULT(readU64Leb128(&initial, "memory initial page count"));
        if (hasMax) {
            CHECK_RESULT(readU64Leb128(&max, "memory max page count"));
        }
    } else {
        uint32_t initial32;
        CHECK_RESULT(readU32Leb128(&initial32, "memory initial page count"));
        initial = initial32;
        if (hasMax) {
            uint32_t max32;
            CHECK_RESULT(readU32Leb128(&max32, "memory max page count"));
            max = max32;
        }
    }

    outPageLimits->has_max = hasMax;
    outPageLimits->is_shared = isShared;
    outPageLimits->is_64 = is64;
    outPageLimits->initial = initial;
    outPageLimits->max = max;

    return Result::Ok;
}

Result BinaryReader::readGlobalHeader(Type *outType, bool *outMutable) {
    Type globalType = Type::Void;
    uint8_t isMutable = 0;
    CHECK_RESULT(readType(&globalType, "global type"));
    ERROR_UNLESS(isConcreteType(globalType), "invalid global type: %#x",
                 static_cast<int>(globalType));

    CHECK_RESULT(readU8(&isMutable, "global mutability"));
    ERROR_UNLESS(isMutable <= 1, "global mutability must be 0 or 1");

    *outType = globalType;
    *outMutable = isMutable;
    return Result::Ok;
}

Result BinaryReader::readAddress(Address *outValue, Index memory,
                                 const char *desc) {
    if (options_.features.memory64_enabled()) {
        return readU64Leb128(outValue, desc);
    }
    uint32_t val;
    Result res = readU32Leb128(&val, desc);
    *outValue = val;
    return res;
}

Result BinaryReader::readFunctionBody(Offset endOffset) {
        CHECK_RESULT(readInstructions(endOffset, "function body"));
        ERROR_UNLESS(state.offset == endOffset,
                     "function body shorter than given size");
        return Result::Ok;
}

Result BinaryReader::readInstructions(Offset endOffset, const char *context) {
    std::stack<Opcode> nestedBlocks;
    while (state.offset < endOffset) {
        Opcode opcode;
        CHECK_RESULT(readOpcode(&opcode, "opcode"));
        CALLBACK(OnOpcode, opcode);
        ERROR_UNLESS_OPCODE_ENABLED(opcode);

        switch (opcode) {
        case Opcode::Unreachable:
            CALLBACK0(OnUnreachableExpr);
            CALLBACK0(OnOpcodeBare);
            break;

        case Opcode::Block: {
            nestedBlocks.push(opcode);
            Type sigType;
            CHECK_RESULT(readType(&sigType, "block signature type"));
            ERROR_UNLESS(isBlockType(sigType),
                         "expected valid block signature type");
            CALLBACK(OnBlockExpr, sigType);
            CALLBACK(OnOpcodeBlockSig, sigType);
            break;
        }

        case Opcode::Loop: {
            nestedBlocks.push(opcode);
            Type sigType;
            CHECK_RESULT(readType(&sigType, "loop signature type"));
            ERROR_UNLESS(isBlockType(sigType),
                         "expected valid block signature type");
            CALLBACK(OnLoopExpr, sigType);
            CALLBACK(OnOpcodeBlockSig, sigType);
            break;
        }

        case Opcode::If: {
            nestedBlocks.push(opcode);
            Type sigType;
            CHECK_RESULT(readType(&sigType, "if signature type"));
            ERROR_UNLESS(isBlockType(sigType),
                         "expected valid block signature type");
            CALLBACK(OnIfExpr, sigType);
            CALLBACK(OnOpcodeBlockSig, sigType);
            break;
        }

        case Opcode::Else:
            ERROR_IF(nestedBlocks.empty() || (nestedBlocks.top() != Opcode::If),
                     "else outside if block");
            CALLBACK0(OnElseExpr);
            CALLBACK0(OnOpcodeBare);
            break;

        case Opcode::SelectT: {
            Index numResults;
            CHECK_RESULT(readCount(&numResults, "num result types"));

            resultTypes.resize(numResults);
            for (Index i = 0; i < numResults; ++i) {
                Type resultType;
                CHECK_RESULT(readType(&resultType, "select result type"));
                ERROR_UNLESS(
                    isConcreteType(resultType),
                    "expected valid select result type (got " PRItypecode ")",
                    WABT_PRINTF_TYPE_CODE(resultType));
                resultTypes[i] = resultType;
            }

            if (numResults) {
                CALLBACK(OnSelectExpr, numResults, resultTypes.data());
                CALLBACK(OnOpcodeType, resultTypes[0]);
            } else {
                CALLBACK(OnSelectExpr, 0, NULL);
                CALLBACK0(OnOpcodeBare);
            }
            break;
        }

        case Opcode::Select:
            CALLBACK(OnSelectExpr, 0, nullptr);
            CALLBACK0(OnOpcodeBare);
            break;

        case Opcode::Br: {
            Index depth;
            CHECK_RESULT(readIndex(&depth, "br depth"));
            CALLBACK(OnBrExpr, depth);
            CALLBACK(OnOpcodeIndex, depth);
            break;
        }

        case Opcode::BrIf: {
            Index depth;
            CHECK_RESULT(readIndex(&depth, "br_if depth"));
            CALLBACK(OnBrIfExpr, depth);
            CALLBACK(OnOpcodeIndex, depth);
            break;
        }

        case Opcode::BrTable: {
            Index numTargets;
            CHECK_RESULT(readCount(&numTargets, "br_table target count"));
            targetDepths.resize(numTargets);

            for (Index i = 0; i < numTargets; ++i) {
                Index targetDepth;
                CHECK_RESULT(readIndex(&targetDepth, "br_table target depth"));
                targetDepths[i] = targetDepth;
            }

            Index defaultTargetDepth;
            CHECK_RESULT(readIndex(&defaultTargetDepth,
                                   "br_table default target depth"));

            Index *targetDepths =
                numTargets ? this->targetDepths.data() : nullptr;

            CALLBACK(OnBrTableExpr, numTargets, targetDepths,
                     defaultTargetDepth);
            break;
        }

        case Opcode::Return:
            CALLBACK0(OnReturnExpr);
            CALLBACK0(OnOpcodeBare);
            break;

        case Opcode::Nop:
            CALLBACK0(OnNopExpr);
            CALLBACK0(OnOpcodeBare);
            break;

        case Opcode::Drop:
            CALLBACK0(OnDropExpr);
            CALLBACK0(OnOpcodeBare);
            break;

        case Opcode::End:
            CALLBACK0(OnEndExpr);
            if (nestedBlocks.empty()) {
                return Result::Ok;
            }
            nestedBlocks.pop();
            break;

        case Opcode::I32Const: {
            uint32_t value;
            CHECK_RESULT(readS32Leb128(&value, "i32.const value"));
            CALLBACK(OnI32ConstExpr, value);
            CALLBACK(OnOpcodeUint32, value);
            break;
        }

        case Opcode::I64Const: {
            uint64_t value;
            CHECK_RESULT(readS64Leb128(&value, "i64.const value"));
            CALLBACK(OnI64ConstExpr, value);
            CALLBACK(OnOpcodeUint64, value);
            break;
        }

        case Opcode::F32Const: {
            uint32_t valueBits = 0;
            CHECK_RESULT(readF32(&valueBits, "f32.const value"));
            CALLBACK(OnF32ConstExpr, valueBits);
            CALLBACK(OnOpcodeF32, valueBits);
            break;
        }

        case Opcode::F64Const: {
            uint64_t valueBits = 0;
            CHECK_RESULT(readF64(&valueBits, "f64.const value"));
            CALLBACK(OnF64ConstExpr, valueBits);
            CALLBACK(OnOpcodeF64, valueBits);
            break;
        }

        case Opcode::V128Const: {
            v128 valueBits;
            ZeroMemory(valueBits);
            CHECK_RESULT(readV128(&valueBits, "v128.const value"));
            CALLBACK(OnV128ConstExpr, valueBits);
            CALLBACK(OnOpcodeV128, valueBits);
            break;
        }

        case Opcode::GlobalGet: {
            Index globalIndex;
            CHECK_RESULT(readIndex(&globalIndex, "global.get global index"));
            CALLBACK(OnGlobalGetExpr, globalIndex);
            CALLBACK(OnOpcodeIndex, globalIndex);
            break;
        }

        case Opcode::LocalGet: {
            Index localIndex;
            CHECK_RESULT(readIndex(&localIndex, "local.get local index"));
            CALLBACK(OnLocalGetExpr, localIndex);
            CALLBACK(OnOpcodeIndex, localIndex);
            break;
        }

        case Opcode::GlobalSet: {
            Index globalIndex;
            CHECK_RESULT(readIndex(&globalIndex, "global.set global index"));
            CALLBACK(OnGlobalSetExpr, globalIndex);
            CALLBACK(OnOpcodeIndex, globalIndex);
            break;
        }

        case Opcode::LocalSet: {
            Index localIndex;
            CHECK_RESULT(readIndex(&localIndex, "local.set local index"));
            CALLBACK(OnLocalSetExpr, localIndex);
            CALLBACK(OnOpcodeIndex, localIndex);
            break;
        }

        case Opcode::Call: {
            Index funcIndex;
            CHECK_RESULT(readIndex(&funcIndex, "call function index"));
            CALLBACK(OnCallExpr, funcIndex);
            CALLBACK(OnOpcodeIndex, funcIndex);
            break;
        }

        case Opcode::CallIndirect: {
            Index sigIndex;
            CHECK_RESULT(readIndex(&sigIndex, "call_indirect signature index"));
            Index tableIndex = 0;
            if (options_.features.reference_types_enabled()) {
                CHECK_RESULT(
                    readIndex(&tableIndex, "call_indirect table index"));
            } else {
                uint8_t reserved;
                CHECK_RESULT(readU8(&reserved, "call_indirect reserved"));
                ERROR_UNLESS(reserved == 0,
                             "call_indirect reserved value must be 0");
            }
            CALLBACK(OnCallIndirectExpr, sigIndex, tableIndex);
            CALLBACK(OnOpcodeUint32Uint32, sigIndex, tableIndex);
            break;
        }

        case Opcode::ReturnCall: {
            Index funcIndex;
            CHECK_RESULT(readIndex(&funcIndex, "return_call"));
            CALLBACK(OnReturnCallExpr, funcIndex);
            CALLBACK(OnOpcodeIndex, funcIndex);
            break;
        }

        case Opcode::ReturnCallIndirect: {
            Index sigIndex;
            CHECK_RESULT(readIndex(&sigIndex, "return_call_indirect"));
            Index tableIndex = 0;
            if (options_.features.reference_types_enabled()) {
                CHECK_RESULT(
                    readIndex(&tableIndex, "return_call_indirect table index"));
            } else {
                uint8_t reserved;
                CHECK_RESULT(
                    readU8(&reserved, "return_call_indirect reserved"));
                ERROR_UNLESS(reserved == 0,
                             "return_call_indirect reserved value must be 0");
            }
            CALLBACK(OnReturnCallIndirectExpr, sigIndex, tableIndex);
            CALLBACK(OnOpcodeUint32Uint32, sigIndex, tableIndex);
            break;
        }

        case Opcode::LocalTee: {
            Index localIndex;
            CHECK_RESULT(readIndex(&localIndex, "local.tee local index"));
            CALLBACK(OnLocalTeeExpr, localIndex);
            CALLBACK(OnOpcodeIndex, localIndex);
            break;
        }

        case Opcode::I32Load8S:
        case Opcode::I32Load8U:
        case Opcode::I32Load16S:
        case Opcode::I32Load16U:
        case Opcode::I64Load8S:
        case Opcode::I64Load8U:
        case Opcode::I64Load16S:
        case Opcode::I64Load16U:
        case Opcode::I64Load32S:
        case Opcode::I64Load32U:
        case Opcode::I32Load:
        case Opcode::I64Load:
        case Opcode::F32Load:
        case Opcode::F64Load:
        case Opcode::V128Load:
        case Opcode::V128Load8X8S:
        case Opcode::V128Load8X8U:
        case Opcode::V128Load16X4S:
        case Opcode::V128Load16X4U:
        case Opcode::V128Load32X2S:
        case Opcode::V128Load32X2U: {
            Address alignmentLog2;
            Index memidx;
            Address offset;
            CHECK_RESULT(readMemLocation(&alignmentLog2, &memidx, &offset,
                                         "load alignment", "load memidx",
                                         "load offset"));
            CALLBACK(OnLoadExpr, opcode, memidx, alignmentLog2, offset);
            CHECK_RESULT(callbackMemLocation(&alignmentLog2, &memidx, &offset));
            break;
        }

        case Opcode::I32Store8:
        case Opcode::I32Store16:
        case Opcode::I64Store8:
        case Opcode::I64Store16:
        case Opcode::I64Store32:
        case Opcode::I32Store:
        case Opcode::I64Store:
        case Opcode::F32Store:
        case Opcode::F64Store:
        case Opcode::V128Store: {
            Address alignmentLog2;
            Index memidx;
            Address offset;
            CHECK_RESULT(readMemLocation(&alignmentLog2, &memidx, &offset,
                                         "store alignment", "store memidx",
                                         "store offset"));
            CALLBACK(OnStoreExpr, opcode, memidx, alignmentLog2, offset);
            CHECK_RESULT(callbackMemLocation(&alignmentLog2, &memidx, &offset));
            break;
        }

        case Opcode::MemorySize: {
            Index memidx = 0;
            if (!options_.features.multi_memory_enabled()) {
                uint8_t reserved;
                CHECK_RESULT(readU8(&reserved, "memory.size reserved"));
                ERROR_UNLESS(reserved == 0,
                             "memory.size reserved value must be 0");
            } else {
                CHECK_RESULT(readMemidx(&memidx, "memory.size memidx"));
            }
            CALLBACK(OnMemorySizeExpr, memidx);
            CALLBACK(OnOpcodeUint32, memidx);
            break;
        }

        case Opcode::MemoryGrow: {
            Index memidx = 0;
            if (!options_.features.multi_memory_enabled()) {
                uint8_t reserved;
                CHECK_RESULT(readU8(&reserved, "memory.grow reserved"));
                ERROR_UNLESS(reserved == 0,
                             "memory.grow reserved value must be 0");
            } else {
                CHECK_RESULT(readMemidx(&memidx, "memory.grow memidx"));
            }
            CALLBACK(OnMemoryGrowExpr, memidx);
            CALLBACK(OnOpcodeUint32, memidx);
            break;
        }

        case Opcode::I32Add:
        case Opcode::I32Sub:
        case Opcode::I32Mul:
        case Opcode::I32DivS:
        case Opcode::I32DivU:
        case Opcode::I32RemS:
        case Opcode::I32RemU:
        case Opcode::I32And:
        case Opcode::I32Or:
        case Opcode::I32Xor:
        case Opcode::I32Shl:
        case Opcode::I32ShrU:
        case Opcode::I32ShrS:
        case Opcode::I32Rotr:
        case Opcode::I32Rotl:
        case Opcode::I64Add:
        case Opcode::I64Sub:
        case Opcode::I64Mul:
        case Opcode::I64DivS:
        case Opcode::I64DivU:
        case Opcode::I64RemS:
        case Opcode::I64RemU:
        case Opcode::I64And:
        case Opcode::I64Or:
        case Opcode::I64Xor:
        case Opcode::I64Shl:
        case Opcode::I64ShrU:
        case Opcode::I64ShrS:
        case Opcode::I64Rotr:
        case Opcode::I64Rotl:
        case Opcode::F32Add:
        case Opcode::F32Sub:
        case Opcode::F32Mul:
        case Opcode::F32Div:
        case Opcode::F32Min:
        case Opcode::F32Max:
        case Opcode::F32Copysign:
        case Opcode::F64Add:
        case Opcode::F64Sub:
        case Opcode::F64Mul:
        case Opcode::F64Div:
        case Opcode::F64Min:
        case Opcode::F64Max:
        case Opcode::F64Copysign:
        case Opcode::I8X16Add:
        case Opcode::I16X8Add:
        case Opcode::I32X4Add:
        case Opcode::I64X2Add:
        case Opcode::I8X16Sub:
        case Opcode::I16X8Sub:
        case Opcode::I32X4Sub:
        case Opcode::I64X2Sub:
        case Opcode::I16X8Mul:
        case Opcode::I32X4Mul:
        case Opcode::I64X2Mul:
        case Opcode::I8X16AddSatS:
        case Opcode::I8X16AddSatU:
        case Opcode::I16X8AddSatS:
        case Opcode::I16X8AddSatU:
        case Opcode::I8X16SubSatS:
        case Opcode::I8X16SubSatU:
        case Opcode::I16X8SubSatS:
        case Opcode::I16X8SubSatU:
        case Opcode::I8X16MinS:
        case Opcode::I16X8MinS:
        case Opcode::I32X4MinS:
        case Opcode::I8X16MinU:
        case Opcode::I16X8MinU:
        case Opcode::I32X4MinU:
        case Opcode::I8X16MaxS:
        case Opcode::I16X8MaxS:
        case Opcode::I32X4MaxS:
        case Opcode::I8X16MaxU:
        case Opcode::I16X8MaxU:
        case Opcode::I32X4MaxU:
        case Opcode::I8X16Shl:
        case Opcode::I16X8Shl:
        case Opcode::I32X4Shl:
        case Opcode::I64X2Shl:
        case Opcode::I8X16ShrS:
        case Opcode::I8X16ShrU:
        case Opcode::I16X8ShrS:
        case Opcode::I16X8ShrU:
        case Opcode::I32X4ShrS:
        case Opcode::I32X4ShrU:
        case Opcode::I64X2ShrS:
        case Opcode::I64X2ShrU:
        case Opcode::V128And:
        case Opcode::V128Or:
        case Opcode::V128Xor:
        case Opcode::F32X4Min:
        case Opcode::F32X4PMin:
        case Opcode::F64X2Min:
        case Opcode::F64X2PMin:
        case Opcode::F32X4Max:
        case Opcode::F32X4PMax:
        case Opcode::F64X2Max:
        case Opcode::F64X2PMax:
        case Opcode::F32X4Add:
        case Opcode::F64X2Add:
        case Opcode::F32X4Sub:
        case Opcode::F64X2Sub:
        case Opcode::F32X4Div:
        case Opcode::F64X2Div:
        case Opcode::F32X4Mul:
        case Opcode::F64X2Mul:
        case Opcode::I8X16Swizzle:
        case Opcode::I8X16NarrowI16X8S:
        case Opcode::I8X16NarrowI16X8U:
        case Opcode::I16X8NarrowI32X4S:
        case Opcode::I16X8NarrowI32X4U:
        case Opcode::V128Andnot:
        case Opcode::I8X16AvgrU:
        case Opcode::I16X8AvgrU:
        case Opcode::I16X8ExtmulLowI8X16S:
        case Opcode::I16X8ExtmulHighI8X16S:
        case Opcode::I16X8ExtmulLowI8X16U:
        case Opcode::I16X8ExtmulHighI8X16U:
        case Opcode::I32X4ExtmulLowI16X8S:
        case Opcode::I32X4ExtmulHighI16X8S:
        case Opcode::I32X4ExtmulLowI16X8U:
        case Opcode::I32X4ExtmulHighI16X8U:
        case Opcode::I64X2ExtmulLowI32X4S:
        case Opcode::I64X2ExtmulHighI32X4S:
        case Opcode::I64X2ExtmulLowI32X4U:
        case Opcode::I64X2ExtmulHighI32X4U:
        case Opcode::I16X8Q15mulrSatS:
        case Opcode::I32X4DotI16X8S:
        case Opcode::I8X16RelaxedSwizzle:
        case Opcode::F32X4RelaxedMin:
        case Opcode::F32X4RelaxedMax:
        case Opcode::F64X2RelaxedMin:
        case Opcode::F64X2RelaxedMax:
        case Opcode::I16X8RelaxedQ15mulrS:
        case Opcode::I16X8DotI8X16I7X16S:
            CALLBACK(OnBinaryExpr, opcode);
            CALLBACK0(OnOpcodeBare);
            break;

        case Opcode::I32Eq:
        case Opcode::I32Ne:
        case Opcode::I32LtS:
        case Opcode::I32LeS:
        case Opcode::I32LtU:
        case Opcode::I32LeU:
        case Opcode::I32GtS:
        case Opcode::I32GeS:
        case Opcode::I32GtU:
        case Opcode::I32GeU:
        case Opcode::I64Eq:
        case Opcode::I64Ne:
        case Opcode::I64LtS:
        case Opcode::I64LeS:
        case Opcode::I64LtU:
        case Opcode::I64LeU:
        case Opcode::I64GtS:
        case Opcode::I64GeS:
        case Opcode::I64GtU:
        case Opcode::I64GeU:
        case Opcode::F32Eq:
        case Opcode::F32Ne:
        case Opcode::F32Lt:
        case Opcode::F32Le:
        case Opcode::F32Gt:
        case Opcode::F32Ge:
        case Opcode::F64Eq:
        case Opcode::F64Ne:
        case Opcode::F64Lt:
        case Opcode::F64Le:
        case Opcode::F64Gt:
        case Opcode::F64Ge:
        case Opcode::I8X16Eq:
        case Opcode::I16X8Eq:
        case Opcode::I32X4Eq:
        case Opcode::I64X2Eq:
        case Opcode::F32X4Eq:
        case Opcode::F64X2Eq:
        case Opcode::I8X16Ne:
        case Opcode::I16X8Ne:
        case Opcode::I32X4Ne:
        case Opcode::I64X2Ne:
        case Opcode::F32X4Ne:
        case Opcode::F64X2Ne:
        case Opcode::I8X16LtS:
        case Opcode::I8X16LtU:
        case Opcode::I16X8LtS:
        case Opcode::I16X8LtU:
        case Opcode::I32X4LtS:
        case Opcode::I32X4LtU:
        case Opcode::I64X2LtS:
        case Opcode::F32X4Lt:
        case Opcode::F64X2Lt:
        case Opcode::I8X16LeS:
        case Opcode::I8X16LeU:
        case Opcode::I16X8LeS:
        case Opcode::I16X8LeU:
        case Opcode::I32X4LeS:
        case Opcode::I32X4LeU:
        case Opcode::I64X2LeS:
        case Opcode::F32X4Le:
        case Opcode::F64X2Le:
        case Opcode::I8X16GtS:
        case Opcode::I8X16GtU:
        case Opcode::I16X8GtS:
        case Opcode::I16X8GtU:
        case Opcode::I32X4GtS:
        case Opcode::I32X4GtU:
        case Opcode::I64X2GtS:
        case Opcode::F32X4Gt:
        case Opcode::F64X2Gt:
        case Opcode::I8X16GeS:
        case Opcode::I8X16GeU:
        case Opcode::I16X8GeS:
        case Opcode::I16X8GeU:
        case Opcode::I32X4GeS:
        case Opcode::I32X4GeU:
        case Opcode::I64X2GeS:
        case Opcode::F32X4Ge:
        case Opcode::F64X2Ge:
            CALLBACK(OnCompareExpr, opcode);
            CALLBACK0(OnOpcodeBare);
            break;

        case Opcode::I32Clz:
        case Opcode::I32Ctz:
        case Opcode::I32Popcnt:
        case Opcode::I64Clz:
        case Opcode::I64Ctz:
        case Opcode::I64Popcnt:
        case Opcode::F32Abs:
        case Opcode::F32Neg:
        case Opcode::F32Ceil:
        case Opcode::F32Floor:
        case Opcode::F32Trunc:
        case Opcode::F32Nearest:
        case Opcode::F32Sqrt:
        case Opcode::F64Abs:
        case Opcode::F64Neg:
        case Opcode::F64Ceil:
        case Opcode::F64Floor:
        case Opcode::F64Trunc:
        case Opcode::F64Nearest:
        case Opcode::F64Sqrt:
        case Opcode::I8X16Splat:
        case Opcode::I16X8Splat:
        case Opcode::I32X4Splat:
        case Opcode::I64X2Splat:
        case Opcode::F32X4Splat:
        case Opcode::F64X2Splat:
        case Opcode::I8X16Neg:
        case Opcode::I16X8Neg:
        case Opcode::I32X4Neg:
        case Opcode::I64X2Neg:
        case Opcode::V128Not:
        case Opcode::V128AnyTrue:
        case Opcode::I8X16Bitmask:
        case Opcode::I16X8Bitmask:
        case Opcode::I32X4Bitmask:
        case Opcode::I64X2Bitmask:
        case Opcode::I8X16AllTrue:
        case Opcode::I16X8AllTrue:
        case Opcode::I32X4AllTrue:
        case Opcode::I64X2AllTrue:
        case Opcode::F32X4Ceil:
        case Opcode::F64X2Ceil:
        case Opcode::F32X4Floor:
        case Opcode::F64X2Floor:
        case Opcode::F32X4Trunc:
        case Opcode::F64X2Trunc:
        case Opcode::F32X4Nearest:
        case Opcode::F64X2Nearest:
        case Opcode::F32X4Neg:
        case Opcode::F64X2Neg:
        case Opcode::F32X4Abs:
        case Opcode::F64X2Abs:
        case Opcode::F32X4Sqrt:
        case Opcode::F64X2Sqrt:
        case Opcode::I16X8ExtendLowI8X16S:
        case Opcode::I16X8ExtendHighI8X16S:
        case Opcode::I16X8ExtendLowI8X16U:
        case Opcode::I16X8ExtendHighI8X16U:
        case Opcode::I32X4ExtendLowI16X8S:
        case Opcode::I32X4ExtendHighI16X8S:
        case Opcode::I32X4ExtendLowI16X8U:
        case Opcode::I32X4ExtendHighI16X8U:
        case Opcode::I64X2ExtendLowI32X4S:
        case Opcode::I64X2ExtendHighI32X4S:
        case Opcode::I64X2ExtendLowI32X4U:
        case Opcode::I64X2ExtendHighI32X4U:
        case Opcode::I8X16Abs:
        case Opcode::I16X8Abs:
        case Opcode::I32X4Abs:
        case Opcode::I64X2Abs:
        case Opcode::I8X16Popcnt:
        case Opcode::I16X8ExtaddPairwiseI8X16S:
        case Opcode::I16X8ExtaddPairwiseI8X16U:
        case Opcode::I32X4ExtaddPairwiseI16X8S:
        case Opcode::I32X4ExtaddPairwiseI16X8U:
        case Opcode::I32X4RelaxedTruncF32X4S:
        case Opcode::I32X4RelaxedTruncF32X4U:
        case Opcode::I32X4RelaxedTruncF64X2SZero:
        case Opcode::I32X4RelaxedTruncF64X2UZero:
            CALLBACK(OnUnaryExpr, opcode);
            CALLBACK0(OnOpcodeBare);
            break;

        case Opcode::V128BitSelect:
        case Opcode::F32X4RelaxedMadd:
        case Opcode::F32X4RelaxedNmadd:
        case Opcode::F64X2RelaxedMadd:
        case Opcode::F64X2RelaxedNmadd:
        case Opcode::I8X16RelaxedLaneSelect:
        case Opcode::I16X8RelaxedLaneSelect:
        case Opcode::I32X4RelaxedLaneSelect:
        case Opcode::I64X2RelaxedLaneSelect:
        case Opcode::I32X4DotI8X16I7X16AddS:
            CALLBACK(OnTernaryExpr, opcode);
            CALLBACK0(OnOpcodeBare);
            break;

        case Opcode::I8X16ExtractLaneS:
        case Opcode::I8X16ExtractLaneU:
        case Opcode::I16X8ExtractLaneS:
        case Opcode::I16X8ExtractLaneU:
        case Opcode::I32X4ExtractLane:
        case Opcode::I64X2ExtractLane:
        case Opcode::F32X4ExtractLane:
        case Opcode::F64X2ExtractLane:
        case Opcode::I8X16ReplaceLane:
        case Opcode::I16X8ReplaceLane:
        case Opcode::I32X4ReplaceLane:
        case Opcode::I64X2ReplaceLane:
        case Opcode::F32X4ReplaceLane:
        case Opcode::F64X2ReplaceLane: {
            uint8_t laneVal;
            CHECK_RESULT(readU8(&laneVal, "Lane idx"));
            CALLBACK(OnSimdLaneOpExpr, opcode, laneVal);
            CALLBACK(OnOpcodeUint64, laneVal);
            break;
        }

        case Opcode::I8X16Shuffle: {
            v128 value;
            CHECK_RESULT(readV128(&value, "Lane idx [16]"));
            CALLBACK(OnSimdShuffleOpExpr, opcode, value);
            CALLBACK(OnOpcodeV128, value);
            break;
        }

        case Opcode::V128Load8Splat:
        case Opcode::V128Load16Splat:
        case Opcode::V128Load32Splat:
        case Opcode::V128Load64Splat: {
            Address alignmentLog2;
            Index memidx;
            Address offset;
            CHECK_RESULT(readMemLocation(&alignmentLog2, &memidx, &offset,
                                         "load alignment", "load memidx",
                                         "load offset"));
            CALLBACK(OnLoadSplatExpr, opcode, memidx, alignmentLog2, offset);
            CHECK_RESULT(callbackMemLocation(&alignmentLog2, &memidx, &offset));
            break;
        }
        case Opcode::V128Load8Lane:
        case Opcode::V128Load16Lane:
        case Opcode::V128Load32Lane:
        case Opcode::V128Load64Lane: {
            Address alignmentLog2;
            Index memidx;
            Address offset;
            uint8_t laneVal;
            CHECK_RESULT(readMemLocation(&alignmentLog2, &memidx, &offset,
                                         "load alignment", "load memidx",
                                         "load offset", &laneVal));
            CALLBACK(OnSimdLoadLaneExpr, opcode, memidx, alignmentLog2, offset,
                     laneVal);
            CHECK_RESULT(callbackMemLocation(&alignmentLog2, &memidx, &offset,
                                             &laneVal));
            break;
        }
        case Opcode::V128Store8Lane:
        case Opcode::V128Store16Lane:
        case Opcode::V128Store32Lane:
        case Opcode::V128Store64Lane: {
            Address alignmentLog2;
            Index memidx;
            Address offset;
            uint8_t laneVal;
            CHECK_RESULT(readMemLocation(&alignmentLog2, &memidx, &offset,
                                         "store alignment", "store memidx",
                                         "store offset", &laneVal));
            CALLBACK(OnSimdStoreLaneExpr, opcode, memidx, alignmentLog2, offset,
                     laneVal);
            CHECK_RESULT(callbackMemLocation(&alignmentLog2, &memidx, &offset,
                                             &laneVal));
            break;
        }
        case Opcode::V128Load32Zero:
        case Opcode::V128Load64Zero: {
            Address alignmentLog2;
            Index memidx;
            Address offset;
            CHECK_RESULT(readMemLocation(&alignmentLog2, &memidx, &offset,
                                         "load alignment", "load memidx",
                                         "load offset"));
            CALLBACK(OnLoadZeroExpr, opcode, memidx, alignmentLog2, offset);
            CHECK_RESULT(callbackMemLocation(&alignmentLog2, &memidx, &offset));
            break;
        }
        case Opcode::I32TruncF32S:
        case Opcode::I32TruncF64S:
        case Opcode::I32TruncF32U:
        case Opcode::I32TruncF64U:
        case Opcode::I32WrapI64:
        case Opcode::I64TruncF32S:
        case Opcode::I64TruncF64S:
        case Opcode::I64TruncF32U:
        case Opcode::I64TruncF64U:
        case Opcode::I64ExtendI32S:
        case Opcode::I64ExtendI32U:
        case Opcode::F32ConvertI32S:
        case Opcode::F32ConvertI32U:
        case Opcode::F32ConvertI64S:
        case Opcode::F32ConvertI64U:
        case Opcode::F32DemoteF64:
        case Opcode::F32ReinterpretI32:
        case Opcode::F64ConvertI32S:
        case Opcode::F64ConvertI32U:
        case Opcode::F64ConvertI64S:
        case Opcode::F64ConvertI64U:
        case Opcode::F64PromoteF32:
        case Opcode::F64ReinterpretI64:
        case Opcode::I32ReinterpretF32:
        case Opcode::I64ReinterpretF64:
        case Opcode::I32Eqz:
        case Opcode::I64Eqz:
        case Opcode::F32X4ConvertI32X4S:
        case Opcode::F32X4ConvertI32X4U:
        case Opcode::I32X4TruncSatF32X4S:
        case Opcode::I32X4TruncSatF32X4U:
        case Opcode::F32X4DemoteF64X2Zero:
        case Opcode::F64X2PromoteLowF32X4:
        case Opcode::I32X4TruncSatF64X2SZero:
        case Opcode::I32X4TruncSatF64X2UZero:
        case Opcode::F64X2ConvertLowI32X4S:
        case Opcode::F64X2ConvertLowI32X4U:
            CALLBACK(OnConvertExpr, opcode);
            CALLBACK0(OnOpcodeBare);
            break;

        case Opcode::Try: {
            nestedBlocks.push(opcode);
            Type sigType;
            CHECK_RESULT(readType(&sigType, "try signature type"));
            ERROR_UNLESS(isBlockType(sigType),
                         "expected valid block signature type");
            CALLBACK(OnTryExpr, sigType);
            CALLBACK(OnOpcodeBlockSig, sigType);
            break;
        }

        case Opcode::Catch: {
            Index index;
            CHECK_RESULT(readIndex(&index, "tag index"));
            CALLBACK(OnCatchExpr, index);
            CALLBACK(OnOpcodeIndex, index);
            break;
        }

        case Opcode::CatchAll: {
            CALLBACK(OnCatchAllExpr);
            CALLBACK(OnOpcodeBare);
            break;
        }

        case Opcode::Delegate: {
            ERROR_IF(nestedBlocks.empty() ||
                         (nestedBlocks.top() != Opcode::Try),
                     "delegate outside try block");
            nestedBlocks.pop();
            Index index;
            CHECK_RESULT(readIndex(&index, "depth"));
            CALLBACK(OnDelegateExpr, index);
            CALLBACK(OnOpcodeIndex, index);
            break;
        }

        case Opcode::Rethrow: {
            Index depth;
            CHECK_RESULT(readIndex(&depth, "catch depth"));
            CALLBACK(OnRethrowExpr, depth);
            CALLBACK(OnOpcodeIndex, depth);
            break;
        }

        case Opcode::Throw: {
            Index index;
            CHECK_RESULT(readIndex(&index, "tag index"));
            CALLBACK(OnThrowExpr, index);
            CALLBACK(OnOpcodeIndex, index);
            break;
        }

        case Opcode::I32Extend8S:
        case Opcode::I32Extend16S:
        case Opcode::I64Extend8S:
        case Opcode::I64Extend16S:
        case Opcode::I64Extend32S:
            CALLBACK(OnUnaryExpr, opcode);
            CALLBACK0(OnOpcodeBare);
            break;

        case Opcode::I32TruncSatF32S:
        case Opcode::I32TruncSatF32U:
        case Opcode::I32TruncSatF64S:
        case Opcode::I32TruncSatF64U:
        case Opcode::I64TruncSatF32S:
        case Opcode::I64TruncSatF32U:
        case Opcode::I64TruncSatF64S:
        case Opcode::I64TruncSatF64U:
            CALLBACK(OnConvertExpr, opcode);
            CALLBACK0(OnOpcodeBare);
            break;

        case Opcode::MemoryAtomicNotify: {
            Address alignmentLog2;
            Index memidx;
            Address offset;
            CHECK_RESULT(readMemLocation(&alignmentLog2, &memidx, &offset,
                                         "notify alignment", "notify memidx",
                                         "notify offset"));
            CALLBACK(OnAtomicNotifyExpr, opcode, memidx, alignmentLog2, offset);
            CHECK_RESULT(callbackMemLocation(&alignmentLog2, &memidx, &offset));
            break;
        }

        case Opcode::MemoryAtomicWait32:
        case Opcode::MemoryAtomicWait64: {
            Address alignmentLog2;
            Index memidx;
            Address offset;
            CHECK_RESULT(readMemLocation(&alignmentLog2, &memidx, &offset,
                                         "wait alignment", "wait memidx",
                                         "wait offset"));
            CALLBACK(OnAtomicWaitExpr, opcode, memidx, alignmentLog2, offset);
            CHECK_RESULT(callbackMemLocation(&alignmentLog2, &memidx, &offset));
            break;
        }

        case Opcode::AtomicFence: {
            uint8_t consistencyModel;
            CHECK_RESULT(readU8(&consistencyModel, "consistency model"));
            ERROR_UNLESS(consistencyModel == 0,
                         "atomic.fence consistency model must be 0");
            CALLBACK(OnAtomicFenceExpr, consistencyModel);
            CALLBACK(OnOpcodeUint32, consistencyModel);
            break;
        }

        case Opcode::I32AtomicLoad8U:
        case Opcode::I32AtomicLoad16U:
        case Opcode::I64AtomicLoad8U:
        case Opcode::I64AtomicLoad16U:
        case Opcode::I64AtomicLoad32U:
        case Opcode::I32AtomicLoad:
        case Opcode::I64AtomicLoad: {
            Address alignmentLog2;
            Index memidx;
            Address offset;
            CHECK_RESULT(readMemLocation(&alignmentLog2, &memidx, &offset,
                                         "load alignment", "load memidx",
                                         "load offset"));
            CALLBACK(OnAtomicLoadExpr, opcode, memidx, alignmentLog2, offset);
            CHECK_RESULT(callbackMemLocation(&alignmentLog2, &memidx, &offset));
            break;
        }

        case Opcode::I32AtomicStore8:
        case Opcode::I32AtomicStore16:
        case Opcode::I64AtomicStore8:
        case Opcode::I64AtomicStore16:
        case Opcode::I64AtomicStore32:
        case Opcode::I32AtomicStore:
        case Opcode::I64AtomicStore: {
            Address alignmentLog2;
            Index memidx;
            Address offset;
            CHECK_RESULT(readMemLocation(&alignmentLog2, &memidx, &offset,
                                         "store alignment", "store memidx",
                                         "store offset"));
            CALLBACK(OnAtomicStoreExpr, opcode, memidx, alignmentLog2, offset);
            CHECK_RESULT(callbackMemLocation(&alignmentLog2, &memidx, &offset));
            break;
        }

        case Opcode::I32AtomicRmwAdd:
        case Opcode::I64AtomicRmwAdd:
        case Opcode::I32AtomicRmw8AddU:
        case Opcode::I32AtomicRmw16AddU:
        case Opcode::I64AtomicRmw8AddU:
        case Opcode::I64AtomicRmw16AddU:
        case Opcode::I64AtomicRmw32AddU:
        case Opcode::I32AtomicRmwSub:
        case Opcode::I64AtomicRmwSub:
        case Opcode::I32AtomicRmw8SubU:
        case Opcode::I32AtomicRmw16SubU:
        case Opcode::I64AtomicRmw8SubU:
        case Opcode::I64AtomicRmw16SubU:
        case Opcode::I64AtomicRmw32SubU:
        case Opcode::I32AtomicRmwAnd:
        case Opcode::I64AtomicRmwAnd:
        case Opcode::I32AtomicRmw8AndU:
        case Opcode::I32AtomicRmw16AndU:
        case Opcode::I64AtomicRmw8AndU:
        case Opcode::I64AtomicRmw16AndU:
        case Opcode::I64AtomicRmw32AndU:
        case Opcode::I32AtomicRmwOr:
        case Opcode::I64AtomicRmwOr:
        case Opcode::I32AtomicRmw8OrU:
        case Opcode::I32AtomicRmw16OrU:
        case Opcode::I64AtomicRmw8OrU:
        case Opcode::I64AtomicRmw16OrU:
        case Opcode::I64AtomicRmw32OrU:
        case Opcode::I32AtomicRmwXor:
        case Opcode::I64AtomicRmwXor:
        case Opcode::I32AtomicRmw8XorU:
        case Opcode::I32AtomicRmw16XorU:
        case Opcode::I64AtomicRmw8XorU:
        case Opcode::I64AtomicRmw16XorU:
        case Opcode::I64AtomicRmw32XorU:
        case Opcode::I32AtomicRmwXchg:
        case Opcode::I64AtomicRmwXchg:
        case Opcode::I32AtomicRmw8XchgU:
        case Opcode::I32AtomicRmw16XchgU:
        case Opcode::I64AtomicRmw8XchgU:
        case Opcode::I64AtomicRmw16XchgU:
        case Opcode::I64AtomicRmw32XchgU: {
            Address alignmentLog2;
            Index memidx;
            Address offset;
            CHECK_RESULT(readMemLocation(&alignmentLog2, &memidx, &offset,
                                         "memory alignment", "memory memidx",
                                         "memory offset"));
            CALLBACK(OnAtomicRmwExpr, opcode, memidx, alignmentLog2, offset);
            CHECK_RESULT(callbackMemLocation(&alignmentLog2, &memidx, &offset));
            break;
        }

        case Opcode::I32AtomicRmwCmpxchg:
        case Opcode::I64AtomicRmwCmpxchg:
        case Opcode::I32AtomicRmw8CmpxchgU:
        case Opcode::I32AtomicRmw16CmpxchgU:
        case Opcode::I64AtomicRmw8CmpxchgU:
        case Opcode::I64AtomicRmw16CmpxchgU:
        case Opcode::I64AtomicRmw32CmpxchgU: {
            Address alignmentLog2;
            Index memidx;
            Address offset;
            CHECK_RESULT(readMemLocation(&alignmentLog2, &memidx, &offset,
                                         "memory alignment", "memory memidx",
                                         "memory offset"));
            CALLBACK(OnAtomicRmwCmpxchgExpr, opcode, memidx, alignmentLog2,
                     offset);
            CHECK_RESULT(callbackMemLocation(&alignmentLog2, &memidx, &offset));
            break;
        }

        case Opcode::TableInit: {
            Index segment;
            CHECK_RESULT(readIndex(&segment, "elem segment index"));
            Index tableIndex;
            CHECK_RESULT(readIndex(&tableIndex, "reserved table index"));
            CALLBACK(OnTableInitExpr, segment, tableIndex);
            CALLBACK(OnOpcodeUint32Uint32, segment, tableIndex);
            break;
        }

        case Opcode::MemoryInit: {
            Index segment;
            ERROR_IF(dataCount == kInvalidIndex,
                     "memory.init requires data count section");
            CHECK_RESULT(readIndex(&segment, "elem segment index"));
            Index memidx = 0;
            if (!options_.features.multi_memory_enabled()) {
                uint8_t reserved;
                CHECK_RESULT(readU8(&reserved, "reserved memory index"));
                ERROR_UNLESS(reserved == 0, "reserved value must be 0");
            } else {
                CHECK_RESULT(readMemidx(&memidx, "memory.init memidx"));
            }
            CALLBACK(OnMemoryInitExpr, segment, memidx);
            CALLBACK(OnOpcodeUint32Uint32, segment, memidx);
            break;
        }

        case Opcode::DataDrop:
            ERROR_IF(dataCount == kInvalidIndex,
                     "data.drop requires data count section");
            [[fallthrough]];
        case Opcode::ElemDrop: {
            Index segment;
            CHECK_RESULT(readIndex(&segment, "segment index"));
            if (opcode == Opcode::DataDrop) {
                CALLBACK(OnDataDropExpr, segment);
            } else {
                CALLBACK(OnElemDropExpr, segment);
            }
            CALLBACK(OnOpcodeUint32, segment);
            break;
        }

        case Opcode::MemoryFill: {
            Index memidx = 0;
            if (!options_.features.multi_memory_enabled()) {
                uint8_t reserved;
                CHECK_RESULT(readU8(&reserved, "memory.fill reserved"));
                ERROR_UNLESS(reserved == 0,
                             "memory.fill reserved value must be 0");
            } else {
                CHECK_RESULT(readMemidx(&memidx, "memory.fill memidx"));
            }
            CALLBACK(OnMemoryFillExpr, memidx);
            CALLBACK(OnOpcodeUint32, memidx);
            break;
        }

        case Opcode::MemoryCopy: {
            Index destmemidx = 0;
            Index srcmemidx = 0;
            if (!options_.features.multi_memory_enabled()) {
                uint8_t reserved;
                CHECK_RESULT(readU8(&reserved, "reserved memory index"));
                ERROR_UNLESS(reserved == 0, "reserved value must be 0");
                CHECK_RESULT(readU8(&reserved, "reserved memory index"));
                ERROR_UNLESS(reserved == 0, "reserved value must be 0");
            } else {
                CHECK_RESULT(
                    readMemidx(&destmemidx, "memory.copy destmemindex"));
                CHECK_RESULT(readMemidx(&srcmemidx, "memory.copy srcmemidx"));
            }
            CALLBACK(OnMemoryCopyExpr, destmemidx, srcmemidx);
            CALLBACK(OnOpcodeUint32Uint32, destmemidx, srcmemidx);
            break;
        }

        case Opcode::TableCopy: {
            Index tableDst;
            Index tableSrc;
            CHECK_RESULT(readIndex(&tableDst, "reserved table index"));
            CHECK_RESULT(readIndex(&tableSrc, "table src"));
            CALLBACK(OnTableCopyExpr, tableDst, tableSrc);
            CALLBACK(OnOpcodeUint32Uint32, tableDst, tableSrc);
            break;
        }

        case Opcode::TableGet: {
            Index table;
            CHECK_RESULT(readIndex(&table, "table index"));
            CALLBACK(OnTableGetExpr, table);
            CALLBACK(OnOpcodeUint32, table);
            break;
        }

        case Opcode::TableSet: {
            Index table;
            CHECK_RESULT(readIndex(&table, "table index"));
            CALLBACK(OnTableSetExpr, table);
            CALLBACK(OnOpcodeUint32, table);
            break;
        }

        case Opcode::TableGrow: {
            Index table;
            CHECK_RESULT(readIndex(&table, "table index"));
            CALLBACK(OnTableGrowExpr, table);
            CALLBACK(OnOpcodeUint32, table);
            break;
        }

        case Opcode::TableSize: {
            Index table;
            CHECK_RESULT(readIndex(&table, "table index"));
            CALLBACK(OnTableSizeExpr, table);
            CALLBACK(OnOpcodeUint32, table);
            break;
        }

        case Opcode::TableFill: {
            Index table;
            CHECK_RESULT(readIndex(&table, "table index"));
            CALLBACK(OnTableFillExpr, table);
            CALLBACK(OnOpcodeUint32, table);
            break;
        }

        case Opcode::RefFunc: {
            Index func;
            CHECK_RESULT(readIndex(&func, "func index"));
            CALLBACK(OnRefFuncExpr, func);
            CALLBACK(OnOpcodeUint32, func);
            break;
        }

        case Opcode::RefNull: {
            Type type;
            CHECK_RESULT(readRefType(&type, "ref.null type"));
            CALLBACK(OnRefNullExpr, type);
            CALLBACK(OnOpcodeType, type);
            break;
        }

        case Opcode::RefIsNull:
            CALLBACK(OnRefIsNullExpr);
            CALLBACK(OnOpcodeBare);
            break;

        case Opcode::CallRef:
            CALLBACK(OnCallRefExpr);
            CALLBACK(OnOpcodeBare);
            break;

        default:
            return ReportUnexpectedOpcode(opcode);
        }
    }

    PrintError("%s must end with END opcode", context);
    return Result::Error;
}

Result BinaryReader::readNameSection(Offset sectionSize) {
    CALLBACK(BeginNamesSection, sectionSize);
    Index i = 0;
    uint32_t previousSubsectionType = 0;
    while (state.offset < readEnd) {
        uint32_t nameType;
        Offset subsectionSize;
        CHECK_RESULT(readU32Leb128(&nameType, "name type"));
        if (i != 0) {
            ERROR_UNLESS(nameType != previousSubsectionType,
                         "duplicate sub-section");
            ERROR_UNLESS(nameType >= previousSubsectionType,
                         "out-of-order sub-section");
        }
        previousSubsectionType = nameType;
        CHECK_RESULT(readOffset(&subsectionSize, "subsection size"));
        size_t subsectionEnd = state.offset + subsectionSize;
        ERROR_UNLESS(subsectionEnd <= readEnd,
                     "invalid sub-section size: extends past end");
        ReadEndRestoreGuard guard(this);
        readEnd = subsectionEnd;

        NameSectionSubsection type =
            static_cast<NameSectionSubsection>(nameType);
        if (type <= NameSectionSubsection::Last) {
            CALLBACK(OnNameSubsection, i, type, subsectionSize);
        }

        switch (type) {
        case NameSectionSubsection::Module:
            CALLBACK(OnModuleNameSubsection, i, nameType, subsectionSize);
            if (subsectionSize) {
                StringRef name;
                CHECK_RESULT(readStr(&name, "module name"));
                CALLBACK(OnModuleName, name);
            }
            break;
        case NameSectionSubsection::Function:
            CALLBACK(OnFunctionNameSubsection, i, nameType, subsectionSize);
            if (subsectionSize) {
                Index numNames;
                CHECK_RESULT(readCount(&numNames, "name count"));
                CALLBACK(OnFunctionNamesCount, numNames);
                Index lastFunctionIndex = kInvalidIndex;

                for (Index j = 0; j < numNames; ++j) {
                    Index functionIndex;
                    StringRef functionName;

                    CHECK_RESULT(readIndex(&functionIndex, "function index"));
                    ERROR_UNLESS(functionIndex != lastFunctionIndex,
                                 "duplicate function name: %u", functionIndex);
                    ERROR_UNLESS(lastFunctionIndex == kInvalidIndex ||
                                     functionIndex > lastFunctionIndex,
                                 "function index out of order: %u",
                                 functionIndex);
                    lastFunctionIndex = functionIndex;
                    ERROR_UNLESS(functionIndex < numTotalFuncs(),
                                 "invalid function index: %" PRIindex,
                                 functionIndex);
                    CHECK_RESULT(readStr(&functionName, "function name"));
                    CALLBACK(OnFunctionName, functionIndex, functionName);
                }
            }
            break;
        case NameSectionSubsection::Local:
            CALLBACK(OnLocalNameSubsection, i, nameType, subsectionSize);
            if (subsectionSize) {
                Index numFuncs;
                CHECK_RESULT(readCount(&numFuncs, "function count"));
                CALLBACK(OnLocalNameFunctionCount, numFuncs);
                Index lastFunctionIndex = kInvalidIndex;
                for (Index j = 0; j < numFuncs; ++j) {
                    Index functionIndex;
                    CHECK_RESULT(readIndex(&functionIndex, "function index"));
                    ERROR_UNLESS(functionIndex < numTotalFuncs(),
                                 "invalid function index: %u", functionIndex);
                    ERROR_UNLESS(lastFunctionIndex == kInvalidIndex ||
                                     functionIndex > lastFunctionIndex,
                                 "locals function index out of order: %u",
                                 functionIndex);
                    lastFunctionIndex = functionIndex;
                    Index numLocals;
                    CHECK_RESULT(readCount(&numLocals, "local count"));
                    CALLBACK(OnLocalNameLocalCount, functionIndex, numLocals);
                    Index lastLocalIndex = kInvalidIndex;
                    for (Index k = 0; k < numLocals; ++k) {
                        Index localIndex;
                        StringRef localName;

                        CHECK_RESULT(readIndex(&localIndex, "named index"));
                        ERROR_UNLESS(localIndex != lastLocalIndex,
                                     "duplicate local index: %u", localIndex);
                        ERROR_UNLESS(lastLocalIndex == kInvalidIndex ||
                                         localIndex > lastLocalIndex,
                                     "local index out of order: %u",
                                     localIndex);
                        lastLocalIndex = localIndex;
                        CHECK_RESULT(readStr(&localName, "name"));
                        CALLBACK(OnLocalName, functionIndex, localIndex,
                                 localName);
                    }
                }
            }
            break;
        case NameSectionSubsection::Label:
            // TODO(sbc): Implement label names. These are slightly more
            // complicated since they refer to offsets in the code section /
            // instruction stream.
            state.offset = subsectionEnd;
            break;
        case NameSectionSubsection::Type:
        case NameSectionSubsection::Table:
        case NameSectionSubsection::Memory:
        case NameSectionSubsection::Global:
        case NameSectionSubsection::ElemSegment:
        case NameSectionSubsection::DataSegment:
        case NameSectionSubsection::Tag:
            if (subsectionSize) {
                Index numNames;
                CHECK_RESULT(readCount(&numNames, "name count"));
                CALLBACK(OnNameCount, numNames);
                for (Index j = 0; j < numNames; ++j) {
                    Index index;
                    StringRef name;

                    CHECK_RESULT(readIndex(&index, "index"));
                    CHECK_RESULT(readStr(&name, "name"));
                    CALLBACK(OnNameEntry, type, index, name);
                }
            }
            state.offset = subsectionEnd;
            break;
        default:
            // Unknown subsection, skip it.
            state.offset = subsectionEnd;
            break;
        }
        ++i;
        ERROR_UNLESS(state.offset == subsectionEnd,
                     "unfinished sub-section (expected end: 0x PRIzx )");
    }
    CALLBACK0(EndNamesSection);
    return Result::Ok;
}

Result BinaryReader::readRelocSection(Offset sectionSize) {
    CALLBACK(BeginRelocSection, sectionSize);
    uint32_t sectionIndex;
    CHECK_RESULT(readU32Leb128(&sectionIndex, "section index"));
    Index numRelocs;
    CHECK_RESULT(readCount(&numRelocs, "relocation count"));
    CALLBACK(OnRelocCount, numRelocs, sectionIndex);
    for (Index i = 0; i < numRelocs; ++i) {
        Offset offset;
        Index index;
        uint32_t relocType, addend = 0;
        CHECK_RESULT(readU32Leb128(&relocType, "relocation type"));
        CHECK_RESULT(readOffset(&offset, "offset"));
        CHECK_RESULT(readIndex(&index, "index"));
        RelocType type = static_cast<RelocType>(relocType);
        switch (type) {
        case RelocType::MemoryAddressLEB:
        case RelocType::MemoryAddressLEB64:
        case RelocType::MemoryAddressSLEB:
        case RelocType::MemoryAddressSLEB64:
        case RelocType::MemoryAddressRelSLEB:
        case RelocType::MemoryAddressRelSLEB64:
        case RelocType::MemoryAddressI32:
        case RelocType::MemoryAddressI64:
        case RelocType::FunctionOffsetI32:
        case RelocType::SectionOffsetI32:
        case RelocType::MemoryAddressTLSSLEB:
        case RelocType::MemoryAddressTLSI32:
            CHECK_RESULT(readS32Leb128(&addend, "addend"));
            break;

        case RelocType::FuncIndexLEB:
        case RelocType::TableIndexSLEB:
        case RelocType::TableIndexSLEB64:
        case RelocType::TableIndexI32:
        case RelocType::TableIndexI64:
        case RelocType::TypeIndexLEB:
        case RelocType::GlobalIndexLEB:
        case RelocType::GlobalIndexI32:
        case RelocType::TagIndexLEB:
        case RelocType::TableIndexRelSLEB:
        case RelocType::TableNumberLEB:
            break;

        default:
            PrintError("unknown reloc type: %s", GetRelocTypeName(type));
            return Result::Error;
        }
        CALLBACK(OnReloc, type, offset, index, addend);
    }
    CALLBACK0(EndRelocSection);
    return Result::Ok;
}

Result BinaryReader::readDylink0Section(Offset sectionSize) {
    CALLBACK(BeginDylinkSection, sectionSize);

    while (state.offset < readEnd) {
        uint32_t dylinkType;
        Offset subsectionSize;
        CHECK_RESULT(readU32Leb128(&dylinkType, "type"));
        CHECK_RESULT(readOffset(&subsectionSize, "subsection size"));
        size_t subsectionEnd = state.offset + subsectionSize;
        ERROR_UNLESS(subsectionEnd <= readEnd,
                     "invalid sub-section size: extends past end");
        ReadEndRestoreGuard guard(this);
        readEnd = subsectionEnd;

        uint32_t count;
        switch (static_cast<DylinkEntryType>(dylinkType)) {
        case DylinkEntryType::MemInfo: {
            uint32_t memSize;
            uint32_t memAlign;
            uint32_t tableSize;
            uint32_t tableAlign;

            CHECK_RESULT(readU32Leb128(&memSize, "mem_size"));
            CHECK_RESULT(readU32Leb128(&memAlign, "mem_align"));
            CHECK_RESULT(readU32Leb128(&tableSize, "table_size"));
            CHECK_RESULT(readU32Leb128(&tableAlign, "table_align"));
            CALLBACK(OnDylinkInfo, memSize, memAlign, tableSize, tableAlign);
            break;
        }
        case DylinkEntryType::Needed:
            CHECK_RESULT(readU32Leb128(&count, "needed_dynlibs"));
            CALLBACK(OnDylinkNeededCount, count);
            while (count--) {
                StringRef soName;
                CHECK_RESULT(readStr(&soName, "dylib so_name"));
                CALLBACK(OnDylinkNeeded, soName);
            }
            break;
        case DylinkEntryType::ImportInfo:
            CHECK_RESULT(readU32Leb128(&count, "count"));
            CALLBACK(OnDylinkImportCount, count);
            for (Index i = 0; i < count; ++i) {
                uint32_t flags = 0;
                StringRef module;
                StringRef field;
                CHECK_RESULT(readStr(&module, "module"));
                CHECK_RESULT(readStr(&field, "field"));
                CHECK_RESULT(readU32Leb128(&flags, "flags"));
                CALLBACK(OnDylinkImport, module, field, flags);
            }
            break;
        case DylinkEntryType::ExportInfo:
            CHECK_RESULT(readU32Leb128(&count, "count"));
            CALLBACK(OnDylinkExportCount, count);
            for (Index i = 0; i < count; ++i) {
                uint32_t flags = 0;
                StringRef name;
                CHECK_RESULT(readStr(&name, "name"));
                CHECK_RESULT(readU32Leb128(&flags, "flags"));
                CALLBACK(OnDylinkExport, name, flags);
            }
            break;
        default:
            // Unknown subsection, skip it.
            state.offset = subsectionEnd;
            break;
        }
        ERROR_UNLESS(state.offset == subsectionEnd,
                     "unfinished sub-section (expected end: 0x PRIzx )");
    }

    CALLBACK0(EndDylinkSection);
    return Result::Ok;
}

Result BinaryReader::readDylinkSection(Offset sectionSize) {
    CALLBACK(BeginDylinkSection, sectionSize);
    uint32_t memSize;
    uint32_t memAlign;
    uint32_t tableSize;
    uint32_t tableAlign;

    CHECK_RESULT(readU32Leb128(&memSize, "mem_size"));
    CHECK_RESULT(readU32Leb128(&memAlign, "mem_align"));
    CHECK_RESULT(readU32Leb128(&tableSize, "table_size"));
    CHECK_RESULT(readU32Leb128(&tableAlign, "table_align"));
    CALLBACK(OnDylinkInfo, memSize, memAlign, tableSize, tableAlign);

    uint32_t count;
    CHECK_RESULT(readU32Leb128(&count, "needed_dynlibs"));
    CALLBACK(OnDylinkNeededCount, count);
    while (count--) {
        StringRef soName;
        CHECK_RESULT(readStr(&soName, "dylib so_name"));
        CALLBACK(OnDylinkNeeded, soName);
    }

    CALLBACK0(EndDylinkSection);
    return Result::Ok;
}

Result BinaryReader::readTargetFeaturesSections(Offset sectionSize) {
    CALLBACK(BeginTargetFeaturesSection, sectionSize);
    uint32_t count;
    CHECK_RESULT(readU32Leb128(&count, "sym count"));
    CALLBACK(OnFeatureCount, count);
    while (count--) {
        uint8_t prefix;
        StringRef name;
        CHECK_RESULT(readU8(&prefix, "prefix"));
        CHECK_RESULT(readStr(&name, "feature name"));
        CALLBACK(OnFeature, prefix, name);
    }
    CALLBACK0(EndTargetFeaturesSection);
    return Result::Ok;
}

Result BinaryReader::readGenericCustomSection(StringRef name,
                                              Offset sectionSize) {
    CALLBACK(BeginGenericCustomSection, sectionSize);
    const void *data;
    Offset customDataSize = readEnd - state.offset;
    CHECK_RESULT(
        readBytesWithSize(&data, customDataSize, "custom section data"));
    CALLBACK(OnGenericCustomSection, name, data, customDataSize);
    CALLBACK0(EndGenericCustomSection);
    return Result::Ok;
}

Result BinaryReader::readLinkingSection(Offset sectionSize) {
    CALLBACK(BeginLinkingSection, sectionSize);
    uint32_t version;
    CHECK_RESULT(readU32Leb128(&version, "version"));
    ERROR_UNLESS(version == 2, "invalid linking metadata version: %u", version);
    while (state.offset < readEnd) {
        uint32_t linkingType;
        Offset subsectionSize;
        CHECK_RESULT(readU32Leb128(&linkingType, "type"));
        CHECK_RESULT(readOffset(&subsectionSize, "subsection size"));
        size_t subsectionEnd = state.offset + subsectionSize;
        ERROR_UNLESS(subsectionEnd <= readEnd,
                     "invalid sub-section size: extends past end");
        ReadEndRestoreGuard guard(this);
        readEnd = subsectionEnd;

        uint32_t count;
        switch (static_cast<LinkingEntryType>(linkingType)) {
        case LinkingEntryType::SymbolTable:
            CHECK_RESULT(readU32Leb128(&count, "sym count"));
            CALLBACK(OnSymbolCount, count);
            for (Index i = 0; i < count; ++i) {
                StringRef name;
                uint32_t flags = 0;
                uint32_t kind = 0;
                CHECK_RESULT(readU32Leb128(&kind, "sym type"));
                CHECK_RESULT(readU32Leb128(&flags, "sym flags"));
                SymbolType symType = static_cast<SymbolType>(kind);
                switch (symType) {
                case SymbolType::Function:
                case SymbolType::Global:
                case SymbolType::Tag:
                case SymbolType::Table: {
                    uint32_t index = 0;
                    CHECK_RESULT(readU32Leb128(&index, "index"));
                    if ((flags & WABT_SYMBOL_FLAG_UNDEFINED) == 0 ||
                        (flags & WABT_SYMBOL_FLAG_EXPLICIT_NAME) != 0)
                        CHECK_RESULT(readStr(&name, "symbol name"));
                    switch (symType) {
                    case SymbolType::Function:
                        CALLBACK(OnFunctionSymbol, i, flags, name, index);
                        break;
                    case SymbolType::Global:
                        CALLBACK(OnGlobalSymbol, i, flags, name, index);
                        break;
                    case SymbolType::Tag:
                        CALLBACK(OnTagSymbol, i, flags, name, index);
                        break;
                    case SymbolType::Table:
                        CALLBACK(OnTableSymbol, i, flags, name, index);
                        break;
                    default:
                        WABT_UNREACHABLE;
                    }
                    break;
                }
                case SymbolType::Data: {
                    uint32_t segment = 0;
                    uint32_t offset = 0;
                    uint32_t size = 0;
                    CHECK_RESULT(readStr(&name, "symbol name"));
                    if ((flags & WABT_SYMBOL_FLAG_UNDEFINED) == 0) {
                        CHECK_RESULT(readU32Leb128(&segment, "segment"));
                        CHECK_RESULT(readU32Leb128(&offset, "offset"));
                        CHECK_RESULT(readU32Leb128(&size, "size"));
                    }
                    CALLBACK(OnDataSymbol, i, flags, name, segment, offset,
                             size);
                    break;
                }
                case SymbolType::Section: {
                    uint32_t index = 0;
                    CHECK_RESULT(readU32Leb128(&index, "index"));
                    CALLBACK(OnSectionSymbol, i, flags, index);
                    break;
                }
                }
            }
            break;
        case LinkingEntryType::SegmentInfo:
            CHECK_RESULT(readU32Leb128(&count, "info count"));
            CALLBACK(OnSegmentInfoCount, count);
            for (Index i = 0; i < count; i++) {
                StringRef name;
                Address alignmentLog2;
                uint32_t flags;
                CHECK_RESULT(readStr(&name, "segment name"));
                CHECK_RESULT(
                    readAlignment(&alignmentLog2, "segment alignment"));
                CHECK_RESULT(readU32Leb128(&flags, "segment flags"));
                CALLBACK(OnSegmentInfo, i, name, alignmentLog2, flags);
            }
            break;
        case LinkingEntryType::InitFunctions:
            CHECK_RESULT(readU32Leb128(&count, "info count"));
            CALLBACK(OnInitFunctionCount, count);
            while (count--) {
                uint32_t priority;
                uint32_t symbol;
                CHECK_RESULT(readU32Leb128(&priority, "priority"));
                CHECK_RESULT(readU32Leb128(&symbol, "symbol index"));
                CALLBACK(OnInitFunction, priority, symbol);
            }
            break;
        case LinkingEntryType::ComdatInfo:
            CHECK_RESULT(readU32Leb128(&count, "count"));
            CALLBACK(OnComdatCount, count);
            while (count--) {
                uint32_t flags;
                uint32_t entryCount;
                StringRef name;
                CHECK_RESULT(readStr(&name, "comdat name"));
                CHECK_RESULT(readU32Leb128(&flags, "flags"));
                CHECK_RESULT(readU32Leb128(&entryCount, "entry count"));
                CALLBACK(OnComdatBegin, name, flags, entryCount);
                while (entryCount--) {
                    uint32_t kind;
                    uint32_t index;
                    CHECK_RESULT(readU32Leb128(&kind, "kind"));
                    CHECK_RESULT(readU32Leb128(&index, "index"));
                    ComdatType comdatType = static_cast<ComdatType>(kind);
                    CALLBACK(OnComdatEntry, comdatType, index);
                }
            }
            break;
        default:
            // Unknown subsection, skip it.
            state.offset = subsectionEnd;
            break;
        }
        ERROR_UNLESS(state.offset == subsectionEnd,
                     "unfinished sub-section (expected end: 0x PRIzx )");
    }
    CALLBACK0(EndLinkingSection);
    return Result::Ok;
}

Result BinaryReader::readTagType(Index *outSigIndex) {
    uint8_t attribute;
    CHECK_RESULT(readU8(&attribute, "tag attribute"));
    ERROR_UNLESS(attribute == 0, "tag attribute must be 0");
    CHECK_RESULT(readIndex(outSigIndex, "tag signature index"));
    return Result::Ok;
}

Result BinaryReader::readTagSection(Offset sectionSize) {
    CALLBACK(BeginTagSection, sectionSize);
    Index numTags;
    CHECK_RESULT(readCount(&numTags, "tag count"));
    CALLBACK(OnTagCount, numTags);

    for (Index i = 0; i < numTags; ++i) {
        Index tagIndex = numTagImports + i;
        Index sigIndex;
        CHECK_RESULT(readTagType(&sigIndex));
        CALLBACK(OnTagType, tagIndex, sigIndex);
    }

    CALLBACK(EndTagSection);
    return Result::Ok;
}

Result BinaryReader::readCodeMetadataSection(StringRef name,
                                             Offset sectionSize) {
    CALLBACK(BeginCodeMetadataSection, name, sectionSize);

    Index numFunctions;
    CHECK_RESULT(readCount(&numFunctions, "function count"));
    CALLBACK(OnCodeMetadataFuncCount, numFunctions);

    Index lastFunctionIndex = kInvalidIndex;
    for (Index i = 0; i < numFunctions; ++i) {
        Index functionIndex;
        CHECK_RESULT(readCount(&functionIndex, "function index"));
        ERROR_UNLESS(functionIndex >= numFuncImports,
                     "function import can't have metadata (got %" PRIindex ")",
                     functionIndex);
        ERROR_UNLESS(functionIndex < numTotalFuncs(),
                     "invalid function index: %" PRIindex, functionIndex);
        ERROR_UNLESS(functionIndex != lastFunctionIndex,
                     "duplicate function index: %" PRIindex, functionIndex);
        ERROR_UNLESS(lastFunctionIndex == kInvalidIndex ||
                         functionIndex > lastFunctionIndex,
                     "function index out of order: %" PRIindex, functionIndex);
        lastFunctionIndex = functionIndex;

        Index numMetadata;
        CHECK_RESULT(readCount(&numMetadata, "metadata instances count"));

        CALLBACK(OnCodeMetadataCount, functionIndex, numMetadata);

        Offset lastCodeOffset = kInvalidOffset;
        for (Index j = 0; j < numMetadata; ++j) {
            Offset codeOffset;
            CHECK_RESULT(readOffset(&codeOffset, "code offset"));
            ERROR_UNLESS(codeOffset != lastCodeOffset,
                         "duplicate code offset:  PRIzx");
            ERROR_UNLESS(lastCodeOffset == kInvalidOffset ||
                             codeOffset > lastCodeOffset,
                         "code offset out of order:  PRIzx");
            lastCodeOffset = codeOffset;

            Address dataSize;
            const void *data;
            CHECK_RESULT(readBytes(&data, &dataSize, "instance data"));
            CALLBACK(OnCodeMetadata, codeOffset, data, dataSize);
        }
    }

    CALLBACK(EndCodeMetadataSection);
    return Result::Ok;
}

Result BinaryReader::readCustomSection(Index sectionIndex, Offset sectionSize) {
    StringRef sectionName;
    CHECK_RESULT(readStr(&sectionName, "section name"));
    CALLBACK(BeginCustomSection, sectionIndex, sectionSize, sectionName);
    ValueRestoreGuard<bool, &BinaryReader::readingCustomSection> guard(this);
    readingCustomSection = true;

    {
        // Backtrack parser when scope ends
        ValueRestoreGuard<BinaryReaderDelegate::State, &BinaryReader::state>
            guard(this);
        CHECK_RESULT(readGenericCustomSection(sectionName, sectionSize));
    }

    if (options_.readDebugNames && sectionName == WABT_BINARY_SECTION_NAME) {
        CHECK_RESULT(readNameSection(sectionSize));
        didReadNamesSection = true;
    } else if (sectionName == WABT_BINARY_SECTION_DYLINK0) {
        CHECK_RESULT(readDylink0Section(sectionSize));
    } else if (sectionName == WABT_BINARY_SECTION_DYLINK) {
        CHECK_RESULT(readDylinkSection(sectionSize));
    } else if (sectionName.rfind(WABT_BINARY_SECTION_RELOC) == 0) {
        // Reloc sections always begin with "reloc."
        CHECK_RESULT(readRelocSection(sectionSize));
    } else if (sectionName == WABT_BINARY_SECTION_TARGET_FEATURES) {
        CHECK_RESULT(readTargetFeaturesSections(sectionSize));
    } else if (sectionName == WABT_BINARY_SECTION_LINKING) {
        CHECK_RESULT(readLinkingSection(sectionSize));
    } else if (options_.features.code_metadata_enabled() &&
               sectionName.find(WABT_BINARY_SECTION_CODE_METADATA) == 0) {
        StringRef metadataName = sectionName;
        metadataName = metadataName.drop_front(
            sizeof(WABT_BINARY_SECTION_CODE_METADATA) - 1);
        CHECK_RESULT(readCodeMetadataSection(metadataName, sectionSize));
    } else {
        // Skip. This is a generic custom section, and is handled above.
        state.offset = readEnd;
    }
    CALLBACK0(EndCustomSection);
    return Result::Ok;
}

Result BinaryReader::readTypeSection(Offset sectionSize) {
    CALLBACK(BeginTypeSection, sectionSize);
    Index numSignatures;
    CHECK_RESULT(readCount(&numSignatures, "type count"));
    CALLBACK(OnTypeCount, numSignatures);

    for (Index i = 0; i < numSignatures; ++i) {
        Type form;
        if (options_.features.gc_enabled()) {
            CHECK_RESULT(readType(&form, "type form"));
        } else {
            uint8_t type;
            CHECK_RESULT(readU8(&type, "type form"));
            ERROR_UNLESS(type == 0x60, "unexpected type form (got %#x)", type);
            form = Type::Func;
        }

        switch (form) {
        case Type::Func: {
            Index numParams;
            CHECK_RESULT(readCount(&numParams, "function param count"));

            paramTypes.resize(numParams);

            for (Index j = 0; j < numParams; ++j) {
                Type paramType;
                CHECK_RESULT(readType(&paramType, "function param type"));
                ERROR_UNLESS(isConcreteType(paramType),
                             "expected valid param type (got " PRItypecode ")",
                             WABT_PRINTF_TYPE_CODE(paramType));
                paramTypes[j] = paramType;
            }

            Index numResults;
            CHECK_RESULT(readCount(&numResults, "function result count"));

            resultTypes.resize(numResults);

            for (Index j = 0; j < numResults; ++j) {
                Type resultType;
                CHECK_RESULT(readType(&resultType, "function result type"));
                ERROR_UNLESS(isConcreteType(resultType),
                             "expected valid result type (got " PRItypecode ")",
                             WABT_PRINTF_TYPE_CODE(resultType));
                resultTypes[j] = resultType;
            }

            Type *paramTypes = numParams ? this->paramTypes.data() : nullptr;
            Type *resultTypes = numResults ? this->resultTypes.data() : nullptr;

            CALLBACK(OnFuncType, i, numParams, paramTypes, numResults,
                     resultTypes);
            break;
        }

        case Type::Struct: {
            ERROR_UNLESS(options_.features.gc_enabled(),
                         "invalid type form: struct not allowed");
            Index numFields;
            CHECK_RESULT(readCount(&numFields, "field count"));

            fields.resize(numFields);
            for (Index j = 0; j < numFields; ++j) {
                CHECK_RESULT(readField(&fields[j]));
            }

            CALLBACK(OnStructType, i, fields.size(), fields.data());
            break;
        }

        case Type::Array: {
            ERROR_UNLESS(options_.features.gc_enabled(),
                         "invalid type form: array not allowed");

            TypeMut field;
            CHECK_RESULT(readField(&field));
            CALLBACK(OnArrayType, i, field);
            break;
        };

        default:
            PrintError("unexpected type form (got " PRItypecode ")",
                       WABT_PRINTF_TYPE_CODE(form));
            return Result::Error;
        }
    }
    CALLBACK0(EndTypeSection);
    return Result::Ok;
}

Result BinaryReader::readImportSection(Offset sectionSize) {
    CALLBACK(BeginImportSection, sectionSize);
    Index numImports;
    CHECK_RESULT(readCount(&numImports, "import count"));
    CALLBACK(OnImportCount, numImports);
    for (Index i = 0; i < numImports; ++i) {
        StringRef moduleName;
        CHECK_RESULT(readStr(&moduleName, "import module name"));
        StringRef fieldName;
        CHECK_RESULT(readStr(&fieldName, "import field name"));

        uint8_t kind;
        CHECK_RESULT(readU8(&kind, "import kind"));
        CALLBACK(OnImport, i, static_cast<ExternalKind>(kind), moduleName,
                 fieldName);
        switch (static_cast<ExternalKind>(kind)) {
        case ExternalKind::Func: {
            Index sigIndex;
            CHECK_RESULT(readIndex(&sigIndex, "import signature index"));
            CALLBACK(OnImportFunc, i, moduleName, fieldName, numFuncImports,
                     sigIndex);
            numFuncImports++;
            break;
        }

        case ExternalKind::Table: {
            Type elemType;
            Limits elemLimits;
            CHECK_RESULT(readTable(&elemType, &elemLimits));
            CALLBACK(OnImportTable, i, moduleName, fieldName, numTableImports,
                     elemType, &elemLimits);
            numTableImports++;
            break;
        }

        case ExternalKind::Memory: {
            Limits pageLimits;
            CHECK_RESULT(readMemory(&pageLimits));
            CALLBACK(OnImportMemory, i, moduleName, fieldName, numMemoryImports,
                     &pageLimits);
            numMemoryImports++;
            break;
        }

        case ExternalKind::Global: {
            Type type;
            bool isMutable;
            CHECK_RESULT(readGlobalHeader(&type, &isMutable));
            CALLBACK(OnImportGlobal, i, moduleName, fieldName, numGlobalImports,
                     type, isMutable);
            numGlobalImports++;
            break;
        }

        case ExternalKind::Tag: {
            ERROR_UNLESS(options_.features.exceptions_enabled(),
                         "invalid import tag kind: exceptions not allowed");
            Index sigIndex;
            CHECK_RESULT(readTagType(&sigIndex));
            CALLBACK(OnImportTag, i, moduleName, fieldName, numTagImports,
                     sigIndex);
            numTagImports++;
            break;
        }

        default:
            PrintError("malformed import kind: %d", kind);
            return Result::Error;
        }
    }

    CALLBACK0(EndImportSection);
    return Result::Ok;
}

Result BinaryReader::readFunctionSection(Offset sectionSize) {
    CALLBACK(BeginFunctionSection, sectionSize);
    CHECK_RESULT(readCount(&numFunctionSignatures, "function signature count"));
    CALLBACK(OnFunctionCount, numFunctionSignatures);
    for (Index i = 0; i < numFunctionSignatures; ++i) {
        Index funcIndex = numFuncImports + i;
        Index sigIndex;
        CHECK_RESULT(readIndex(&sigIndex, "function signature index"));
        CALLBACK(OnFunction, funcIndex, sigIndex);
    }
    CALLBACK0(EndFunctionSection);
    return Result::Ok;
}

Result BinaryReader::readTableSection(Offset sectionSize) {
    CALLBACK(BeginTableSection, sectionSize);
    Index numTables;
    CHECK_RESULT(readCount(&numTables, "table count"));
    CALLBACK(OnTableCount, numTables);
    for (Index i = 0; i < numTables; ++i) {
        Index tableIndex = numTableImports + i;
        Type elemType;
        Limits elemLimits;
        CHECK_RESULT(readTable(&elemType, &elemLimits));
        CALLBACK(OnTable, tableIndex, elemType, &elemLimits);
    }
    CALLBACK0(EndTableSection);
    return Result::Ok;
}

Result BinaryReader::readMemorySection(Offset sectionSize) {
    CALLBACK(BeginMemorySection, sectionSize);
    Index numMemories;
    CHECK_RESULT(readCount(&numMemories, "memory count"));
    CALLBACK(OnMemoryCount, numMemories);
    for (Index i = 0; i < numMemories; ++i) {
        Index memoryIndex = numMemoryImports + i;
        Limits pageLimits;
        CHECK_RESULT(readMemory(&pageLimits));
        CALLBACK(OnMemory, memoryIndex, &pageLimits);
    }
    CALLBACK0(EndMemorySection);
    return Result::Ok;
}

Result BinaryReader::readGlobalSection(Offset sectionSize) {
    CALLBACK(BeginGlobalSection, sectionSize);
    Index numGlobals;
    CHECK_RESULT(readCount(&numGlobals, "global count"));
    CALLBACK(OnGlobalCount, numGlobals);
    for (Index i = 0; i < numGlobals; ++i) {
        Index globalIndex = numGlobalImports + i;
        Type globalType;
        bool isMutable;
        CHECK_RESULT(readGlobalHeader(&globalType, &isMutable));
        CALLBACK(BeginGlobal, globalIndex, globalType, isMutable);
        CALLBACK(BeginGlobalInitExpr, globalIndex);
        CHECK_RESULT(readInitExpr(globalIndex));
        CALLBACK(EndGlobalInitExpr, globalIndex);
        CALLBACK(EndGlobal, globalIndex);
    }
    CALLBACK0(EndGlobalSection);
    return Result::Ok;
}

Result BinaryReader::readExportSection(Offset sectionSize) {
    CALLBACK(BeginExportSection, sectionSize);
    Index numExports;
    CHECK_RESULT(readCount(&numExports, "export count"));
    CALLBACK(OnExportCount, numExports);
    for (Index i = 0; i < numExports; ++i) {
        StringRef name;
        CHECK_RESULT(readStr(&name, "export item name"));

        ExternalKind kind;
        CHECK_RESULT(readExternalKind(&kind, "export kind"));

        Index itemIndex;
        CHECK_RESULT(readIndex(&itemIndex, "export item index"));
        if (kind == ExternalKind::Tag) {
            ERROR_UNLESS(options_.features.exceptions_enabled(),
                         "invalid export tag kind: exceptions not allowed");
        }

        CALLBACK(OnExport, i, static_cast<ExternalKind>(kind), itemIndex, name);
    }
    CALLBACK0(EndExportSection);
    return Result::Ok;
}

Result BinaryReader::readStartSection(Offset sectionSize) {
    CALLBACK(BeginStartSection, sectionSize);
    Index funcIndex;
    CHECK_RESULT(readIndex(&funcIndex, "start function index"));
    CALLBACK(OnStartFunction, funcIndex);
    CALLBACK0(EndStartSection);
    return Result::Ok;
}

Result BinaryReader::readElemSection(Offset sectionSize) {
    CALLBACK(BeginElemSection, sectionSize);
    Index numElemSegments;
    CHECK_RESULT(readCount(&numElemSegments, "elem segment count"));
    CALLBACK(OnElemSegmentCount, numElemSegments);
    for (Index i = 0; i < numElemSegments; ++i) {
        uint32_t flags;
        CHECK_RESULT(readU32Leb128(&flags, "elem segment flags"));
        ERROR_IF(flags > SegFlagMax, "invalid elem segment flags: %#x", flags);
        Index tableIndex(0);
        if ((flags & (SegPassive | SegExplicitIndex)) == SegExplicitIndex) {
            CHECK_RESULT(readIndex(&tableIndex, "elem segment table index"));
        }
        Type elemType = Type::FuncRef;

        CALLBACK(BeginElemSegment, i, tableIndex, flags);

        if (!(flags & SegPassive)) {
            CALLBACK(BeginElemSegmentInitExpr, i);
            CHECK_RESULT(readInitExpr(i));
            CALLBACK(EndElemSegmentInitExpr, i);
        }

        // For backwards compat we support not declaring the element kind.
        if (flags & (SegPassive | SegExplicitIndex)) {
            if (flags & SegUseElemExprs) {
                CHECK_RESULT(readRefType(&elemType, "table elem type"));
            } else {
                ExternalKind kind;
                CHECK_RESULT(readExternalKind(&kind, "export kind"));
                ERROR_UNLESS(kind == ExternalKind::Func,
                             "segment elem type must be func (%s)",
                             elemType.GetName().c_str());
                elemType = Type::FuncRef;
            }
        }

        CALLBACK(OnElemSegmentElemType, i, elemType);

        Index numElemExprs;
        CHECK_RESULT(readCount(&numElemExprs, "elem count"));

        CALLBACK(OnElemSegmentElemExprCount, i, numElemExprs);
        for (Index j = 0; j < numElemExprs; ++j) {
            CALLBACK(BeginElemExpr, i, j);
            if (flags & SegUseElemExprs) {
                CHECK_RESULT(readInitExpr(j));
            } else {
                Index funcIndex;
                CHECK_RESULT(readIndex(&funcIndex, "elem expr func index"));
                CALLBACK(OnOpcode, Opcode::RefFunc);
                CALLBACK(OnRefFuncExpr, funcIndex);
                CALLBACK(OnOpcodeUint32, funcIndex);
                CALLBACK0(OnEndExpr);
            }
            CALLBACK(EndElemExpr, i, j);
        }
        CALLBACK(EndElemSegment, i);
    }
    CALLBACK0(EndElemSection);
    return Result::Ok;
}

Result BinaryReader::readCodeSection(Offset sectionSize) {
    CALLBACK(BeginCodeSection, sectionSize);
    CHECK_RESULT(readCount(&numFunctionBodies, "function body count"));
    ERROR_UNLESS(numFunctionSignatures == numFunctionBodies,
                 "function signature count != function body count");
    CALLBACK(OnFunctionBodyCount, numFunctionBodies);
    for (Index i = 0; i < numFunctionBodies; ++i) {
        Index funcIndex = numFuncImports + i;
        Offset funcOffset = state.offset;
        state.offset = funcOffset;
        uint32_t bodySize;
        CHECK_RESULT(readU32Leb128(&bodySize, "function body size"));
        Offset bodyStartOffset = state.offset;
        Offset endOffset = bodyStartOffset + bodySize;
        CALLBACK(BeginFunctionBody, funcIndex, bodySize);

        uint64_t totalLocals = 0;
        Index numLocalDecls;
        CHECK_RESULT(readCount(&numLocalDecls, "local declaration count"));
        CALLBACK(OnLocalDeclCount, numLocalDecls);
        for (Index k = 0; k < numLocalDecls; ++k) {
            Index numLocalTypes;
            CHECK_RESULT(readIndex(&numLocalTypes, "local type count"));
            totalLocals += numLocalTypes;
            ERROR_UNLESS(totalLocals <= UINT32_MAX,
                         "local count must be <= 0x%x", UINT32_MAX);
            Type localType;
            CHECK_RESULT(readType(&localType, "local type"));
            ERROR_UNLESS(isConcreteType(localType),
                         "expected valid local type");
            CALLBACK(OnLocalDecl, k, numLocalTypes, localType);
        }

        state.offset = endOffset;

            CALLBACK(EndFunctionBody, funcIndex);
    }
    CALLBACK0(EndCodeSection);
    return Result::Ok;
}

Result BinaryReader::readDataSection(Offset sectionSize) {
    CALLBACK(BeginDataSection, sectionSize);
    Index numDataSegments;
    CHECK_RESULT(readCount(&numDataSegments, "data segment count"));
    CALLBACK(OnDataSegmentCount, numDataSegments);
    // If the DataCount section is not present, then data_count_ will be
    // invalid.
    ERROR_UNLESS(
        dataCount == kInvalidIndex || dataCount == numDataSegments,
        "data segment count does not equal count in DataCount section");
    for (Index i = 0; i < numDataSegments; ++i) {
        uint32_t flags;
        CHECK_RESULT(readU32Leb128(&flags, "data segment flags"));
        ERROR_IF(flags != 0 && !options_.features.bulk_memory_enabled(),
                 "invalid memory index %d: bulk memory not allowed", flags);
        ERROR_IF(flags > SegFlagMax, "invalid data segment flags: %#x", flags);
        Index memoryIndex(0);
        if (flags & SegExplicitIndex) {
            CHECK_RESULT(readIndex(&memoryIndex, "data segment memory index"));
        }
        CALLBACK(BeginDataSegment, i, memoryIndex, flags);
        if (!(flags & SegPassive)) {
            CALLBACK(BeginDataSegmentInitExpr, i);
            CHECK_RESULT(readInitExpr(i));
            CALLBACK(EndDataSegmentInitExpr, i);
        }

        Address dataSize;
        const void *data;
        CHECK_RESULT(readBytes(&data, &dataSize, "data segment data"));
        CALLBACK(OnDataSegmentData, i, data, dataSize);
        CALLBACK(EndDataSegment, i);
    }
    CALLBACK0(EndDataSection);
    return Result::Ok;
}

Result BinaryReader::readDataCountSection(Offset sectionSize) {
    CALLBACK(BeginDataCountSection, sectionSize);
    Index dataCount;
    CHECK_RESULT(readIndex(&dataCount, "data count"));
    CALLBACK(OnDataCount, dataCount);
    CALLBACK0(EndDataCountSection);
    dataCount = dataCount;
    return Result::Ok;
}

Result BinaryReader::readSections(const ReadSectionsOptions &options) {
    Result result = Result::Ok;
    Index sectionIndex = 0;
    bool seenSectionCode[static_cast<int>(BinarySection::Last) + 1] = {false};

    for (; state.offset < state.size; ++sectionIndex) {
        uint8_t sectionCode;
        Offset sectionSize;
        CHECK_RESULT(readU8(&sectionCode, "section code"));
        CHECK_RESULT(readOffset(&sectionSize, "section size"));
        ReadEndRestoreGuard guard(this);
        readEnd = state.offset + sectionSize;
        if (sectionCode >= kBinarySectionCount) {
            PrintError("invalid section code: %u", sectionCode);
            if (options.stopOnFirstError) {
                return Result::Error;
            }
            // If we don't have to stop on first error, continue reading
            // sections, because although we could not understand the
            // current section, we can continue and correctly parse
            // subsequent sections, so we can give back as much information
            // as we can understand.
            result = Result::Error;
            state.offset = readEnd;
            continue;
        }

        BinarySection section = static_cast<BinarySection>(sectionCode);
        if (section != BinarySection::Custom) {
            if (seenSectionCode[sectionCode]) {
                PrintError("multiple %s sections", GetSectionName(section));
                return Result::Error;
            }
            seenSectionCode[sectionCode] = true;
        }

        ERROR_UNLESS(readEnd <= state.size,
                     "invalid section size: extends past end");

        ERROR_UNLESS(lastKnownSection == BinarySection::Invalid ||
                         section == BinarySection::Custom ||
                         GetSectionOrder(section) >
                             GetSectionOrder(lastKnownSection),
                     "section %s out of order", GetSectionName(section));

        ERROR_UNLESS(!didReadNamesSection || section == BinarySection::Custom,
                     "%s section can not occur after Name section",
                     GetSectionName(section));

        CALLBACK(BeginSection, sectionIndex, section, sectionSize);

        bool stopOnFirstError = options_.stopOnFirstError;
        Result sectionResult = Result::Error;
        switch (section) {
        case BinarySection::Custom:
            sectionResult = readCustomSection(sectionIndex, sectionSize);
            if (options_.failOnCustomSectionError) {
                result |= sectionResult;
            } else {
                stopOnFirstError = false;
            }
            break;
        case BinarySection::Type:
            sectionResult = readTypeSection(sectionSize);
            result |= sectionResult;
            break;
        case BinarySection::Import:
            sectionResult = readImportSection(sectionSize);
            result |= sectionResult;
            break;
        case BinarySection::Function:
            sectionResult = readFunctionSection(sectionSize);
            result |= sectionResult;
            break;
        case BinarySection::Table:
            sectionResult = readTableSection(sectionSize);
            result |= sectionResult;
            break;
        case BinarySection::Memory:
            sectionResult = readMemorySection(sectionSize);
            result |= sectionResult;
            break;
        case BinarySection::Global:
            sectionResult = readGlobalSection(sectionSize);
            result |= sectionResult;
            break;
        case BinarySection::Export:
            sectionResult = readExportSection(sectionSize);
            result |= sectionResult;
            break;
        case BinarySection::Start:
            sectionResult = readStartSection(sectionSize);
            result |= sectionResult;
            break;
        case BinarySection::Elem:
            sectionResult = readElemSection(sectionSize);
            result |= sectionResult;
            break;
        case BinarySection::Code:
            sectionResult = readCodeSection(sectionSize);
            result |= sectionResult;
            break;
        case BinarySection::Data:
            sectionResult = readDataSection(sectionSize);
            result |= sectionResult;
            break;
        case BinarySection::Tag:
            ERROR_UNLESS(options_.features.exceptions_enabled(),
                         "invalid section code: %u",
                         static_cast<unsigned int>(section));
            sectionResult = readTagSection(sectionSize);
            result |= sectionResult;
            break;
        case BinarySection::DataCount:
            ERROR_UNLESS(options_.features.bulk_memory_enabled(),
                         "invalid section code: %u",
                         static_cast<unsigned int>(section));
            sectionResult = readDataCountSection(sectionSize);
            result |= sectionResult;
            break;
        case BinarySection::Invalid:
            WABT_UNREACHABLE;
        }

        if (succeeded(sectionResult) && state.offset != readEnd) {
            PrintError("unfinished section (expected end: 0x PRIzx )");
            sectionResult = Result::Error;
            result |= sectionResult;
        }

        if (failed(sectionResult)) {
            if (stopOnFirstError) {
                return Result::Error;
            }

            // If we're continuing after failing to read this section, move the
            // offset to the expected section end. This way we may be able to
            // read further sections.
            state.offset = readEnd;
        }

        if (section != BinarySection::Custom) {
            lastKnownSection = section;
        }
    }

    return result;
}

Result BinaryReader::readModule(const ReadModuleOptions &options) {
    uint32_t magic = 0;
    CHECK_RESULT(readU32(&magic, "magic"));
    ERROR_UNLESS(magic == WABT_BINARY_MAGIC, "bad magic value");
    uint32_t version = 0;
    CHECK_RESULT(readU32(&version, "version"));
    ERROR_UNLESS(version == WABT_BINARY_VERSION,
                 "bad wasm file version: %#x (expected %#x)", version,
                 WABT_BINARY_VERSION);

    CALLBACK(BeginModule, version);
    CHECK_RESULT(readSections(ReadSectionsOptions{options.stopOnFirstError}));
    // This is checked in ReadCodeSection, but it must be checked at the end
    // too, in case the code section was omitted.
    ERROR_UNLESS(numFunctionSignatures == numFunctionBodies,
                 "function signature count != function body count");
    CALLBACK0(EndModule);

    return Result::Ok;
}

} // end anonymous namespace

Result ReadBinary(const void *data, size_t size, BinaryReaderDelegate *delegate,
                  const ReadBinaryOptions &options) {
    BinaryReader reader(data, size, delegate, options);
    return reader.readModule(
        BinaryReader::ReadModuleOptions{options.stopOnFirstError});
}

} // namespace wabt
