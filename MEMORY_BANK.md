# Webrogue Memory Bank and Graphics Documentation

## Graphics Implementation

Webrogue implements graphics using a multi-layered architecture that abstracts platform-specific APIs while providing high-performance rendering capabilities across Windows, macOS, Linux, Android, and iOS.

### Architecture Layers

1. **High-level Graphics Abstraction** (`webrogue-gfx` crate)
2. **Platform-specific Backend** (`webrogue-gfx-sdl` crate)
3. **Graphics Stream Processing** (`webrogue-gfxstream` crate)
4. **Event System** (Custom event encoding)

### Core Components

#### Graphics Abstraction Layer
- Traits: `ISystem`, `IWindow`, `Interface`
- WITX-defined WebAssembly interface
- Located in `crates/gfx/`

#### SDL Backend
- Implementation of graphics traits using SDL3
- OpenGL ES and Vulkan support
- Located in `crates/gfx-sdl/`

#### Graphics Stream Processing
- Uses Android Emulator (AEMU) technology
- Command buffer processing
- Located in `crates/gfxstream/`

#### Event System
- Custom event encoding/decoding
- Macro-based

### Implementation Details

#### Graphics Initialization Flow

1. **System Setup:**
   ```
   SDLSystem::new() -> Initializes SDL context
   ```

2. **Window Creation:**
   ```
   SDLSystem.make_window() -> Creates SDL window with OpenGL context
   ```

3. **Graphics Stream Initialization:**
   ```
   webrogue_gfxstream_ffi_create_global_state() -> Sets up AEMU graphics
   ```

#### Rendering Loop

1. **Command Submission:**
   - WebAssembly code sends graphics commands
   - Commands are buffered in graphics stream
   - Processed by AEMU graphics system

2. **Presentation:**
   ```
   SDLWindow.present() -> Swaps OpenGL framebuffers
   ```

3. **Event Processing:**
   - SDL events captured and converted
   - Events encoded and sent to WebAssembly
   - Application responds to events

#### Input Handling

Supported Events:
- Mouse button events
- Mouse motion events
- Keyboard key events
- Window resize events
- Text input events

### Key Features

1. **Cross-platform Compatibility**: Works on Windows, macOS, Linux, Android, iOS
2. **Modern Graphics APIs**: Supports OpenGL ES and Vulkan through emulation
3. **Efficient Command Processing**: Uses Android Emulator's streaming technology
4. **Thread Safety**: Dedicated threads for graphics processing
5. **WebAssembly Integration**: Seamless integration through WITX interfaces
6. **Comprehensive Event System**: Handles various input device types

## DynamicMemory Implementation

The Webrogue project has a custom implementation of dynamic memory management in the web crate, specifically in `crates/web/src/memory.rs`. This implementation provides a bridge between WebAssembly's memory management and the host system.

### Key Components

1. **DynamicMemory struct**: An empty struct that implements the `wiggle::DynamicGuestMemory` trait.

2. **External C functions**:
   - `wr_read_memory`: Reads data from memory
   - `wr_write_memory`: Writes data to memory
   - `wr_memory_size`: Gets the size of memory

3. **Implementation of DynamicGuestMemory**:
   - `size()`: Returns the memory size by calling the external `wr_memory_size` function
   - `write()`: Writes data by calling `wr_write_memory`
   - `read()`: Reads data by calling `wr_read_memory`

### Integration with WebAssembly

In `crates/web/src/lib.rs`, the DynamicMemory implementation is used when executing WebAssembly modules:

1. The system parses the WebAssembly module to find memory limits
2. If shared memory is detected, it initializes it with `ffi::wr_make_shared_memory`
3. The WebAssembly module is initialized with the DynamicMemory implementation
4. Functions are executed using the memory management system

### Memory Initialization Process

1. **Parsing**: The WebAssembly module is parsed to determine memory requirements
2. **Limit Detection**: Memory limits are extracted from the module's import section
3. **Shared Memory Setup**: If shared memory is required, it's initialized with specific initial and maximum limits
4. **Execution**: Functions are executed with the DynamicMemory implementation handling memory access

The system uses a macro-based approach (`webrogue_web_macro::wr_web_integration`) to generate the necessary bindings between WebAssembly and the host memory management system.

This implementation allows Webrogue to provide dynamic memory management that works across different platforms while maintaining compatibility with WebAssembly's memory model.

## Memory Bank Functions

The memory bank system provides the following external functions for memory management:

1. `wr_read_memory(modPtr: u32, size: u32, hostPtr: *mut u8)` - Reads data from memory
2. `wr_write_memory(modPtr: u32, size: u32, hostPtr: *const u8)` - Writes data to memory
3. `wr_memory_size() -> u32` - Gets the current memory size
4. `wr_make_shared_memory(initial_pages: u32, max_pages: u32)` - Initializes shared memory with specified limits

## Usage in Webrogue Web Integration

The DynamicMemory implementation is used in the web integration macro to provide memory access for WebAssembly modules:

```rust
let mut memory = wiggle::GuestMemory::Dynamic(Box::new(memory::DynamicMemory {}));
let ret = #abi_func(u, &mut memory #(, #arg_names)*)#await_.unwrap();
```

This allows WebAssembly modules to access memory through the DynamicMemory interface, which translates memory operations to the appropriate host system calls.
