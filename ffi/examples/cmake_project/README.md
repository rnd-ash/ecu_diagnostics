# Sample project for using ecu_diagnostic's FFI bindings in a C project

## Before use
1. in the parent FFI directory, run the following command
```
cargo build
```

This will generate 2 files we need to move into this directory. 
```
ecu_diagnostics/target/<release type>/libecu_diagnostics_ffi.<EXT>
```
Here, `<release type>` will be either debug or release depending on what release type you built.

`EXT` will be different based on the target platform. `.dll` for Windows, `.so` for Linux and `.dylib` for OSX

The second file produced would be
```
ecu_diagnostics/ffi/ecu_diagnostics_ffi.hpp
```

Move the .hpp file to `src/` and the `libecu_diagnostics_ffi` library to the same directory as this README. 

Next, run `cmake .` and `make` in order to produce the executable.
