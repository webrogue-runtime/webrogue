
#include "lld/Common/Driver.h"
#include "lld/Common/ErrorHandler.h"
#include "lld/Common/Memory.h"
#include "llvm/ADT/STLExtras.h"
#include "llvm/ADT/SmallVector.h"
#include "llvm/ADT/Twine.h"
#include <cstdlib>
#include <optional>

using namespace lld;
using namespace llvm;
using namespace llvm::sys;

namespace lld {

// Bypass the crash recovery handler, which is only meant to be used in
// LLD-as-lib scenarios.
int unsafeLldMain(llvm::ArrayRef<const char *> args,
                  llvm::raw_ostream &stdoutOS, llvm::raw_ostream &stderrOS,
                  llvm::ArrayRef<DriverDef> drivers, bool exitEarly);

} // namespace lld

LLD_HAS_DRIVER(coff)
LLD_HAS_DRIVER(elf)
LLD_HAS_DRIVER(mingw)
LLD_HAS_DRIVER(macho)
LLD_HAS_DRIVER(wasm)

extern "C" int webrogue_lld_adapter(int argc, char **argv) {
  ArrayRef<const char *> args(argv, argv + argc);

  return lld::unsafeLldMain(args, llvm::outs(), llvm::errs(), LLD_ALL_DRIVERS,
                            false);
}
