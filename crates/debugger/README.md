This crate is something like Wasmtime's builtin debugger component, but with a few differences:
- Implemented on host side, not as a WebAssembly component
- Works with programs relying on WASI-threads
- Uses traits for IO. There is a `tokio_tcp_connection` to get IO working with plain TCP port, but other implementations may be used

Parts of this code is copy-pasted from Wasmtime's builtin debugger component.
TODO figure out what to do with licenses
