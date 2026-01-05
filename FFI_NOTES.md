# Foreign Function Interface (FFI) Notes

## C Library Integration for I2C

When adding C libraries to interface with the I2C bus, you'll need to set up FFI bindings.

### Required Dependencies

Add these to `Cargo.toml`:

```toml
# FFI bindings for C libraries
bindgen = "0.69"  # For generating Rust bindings from C headers

# Or use cbindgen if generating C headers from Rust
# cbindgen = "0.26"
```

### Typical Setup

1. **Create a `build.rs` build script** to compile C code and generate bindings:

```rust
// build.rs
use std::env;
use std::path::PathBuf;

fn main() {
    // Tell cargo to link the C library
    println!("cargo:rustc-link-lib=your_i2c_library");
    
    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed=wrapper.h");
    
    // Generate bindings
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");
    
    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
```

2. **Create a wrapper header** (`wrapper.h`) that includes the C library headers:

```c
// wrapper.h
#include "your_i2c_library.h"
```

3. **Create a Rust module** to wrap the FFI calls:

```rust
// src/i2c_ffi.rs
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

// Safe wrapper functions
pub fn safe_i2c_function() -> Result<(), String> {
    unsafe {
        let result = unsafe_i2c_function();
        if result == 0 {
            Ok(())
        } else {
            Err("I2C operation failed".to_string())
        }
    }
}
```

### Alternative: Using `cc` crate for C compilation

If you need to compile C code as part of the build:

```toml
[build-dependencies]
cc = "1.0"
bindgen = "0.69"
```

```rust
// build.rs
fn main() {
    cc::Build::new()
        .file("src/i2c_wrapper.c")
        .compile("i2c_wrapper");
    
    // Then generate bindings...
}
```

### Safety Considerations

- Always wrap unsafe FFI calls in safe Rust functions
- Validate all inputs before passing to C functions
- Handle error codes properly
- Document memory ownership and lifetimes
- Use `#[repr(C)]` for structs shared between Rust and C

### Example Integration

Once C libraries are added, update `src/hardware.rs` to use the FFI bindings instead of (or in addition to) `rppal` for I2C operations.
