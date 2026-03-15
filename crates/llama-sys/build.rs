use std::env;
use std::path::PathBuf;

fn main() {
    // 1. Path Management
    // Use CARGO_MANIFEST_DIR to anchor paths absolutely to avoid issues with relative path resolution
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let llama_cpp_root = manifest_dir.join("..").join("..").join("third_party").join("llama.cpp");

    // Allow users to override the source directory via environment variable
    let llama_cpp_path = env::var("LLAMA_SOURCE_DIR")
        .map(PathBuf::from)
        .unwrap_or(llama_cpp_root);

    let include_path = llama_cpp_path.join("include");
    let ggml_include_path = llama_cpp_path.join("ggml").join("include");
    let header_path = include_path.join("llama.h");

    // 2. Metadata for Cargo
    // Re-run build if the header or the build script itself changes
    println!("cargo:rerun-if-changed={}", header_path.display());
    println!("cargo:rerun-if-changed=build.rs");
    // Also watch for environment variable changes that might affect the build
    println!("cargo:rerun-if-env-changed=LLAMA_SOURCE_DIR");


    // 3. Configure and Build via CMake
    let mut config = cmake::Config::new(&llama_cpp_path);
    
    config
        .define("LLAMA_STATIC", "ON")
        .define("LLAMA_BUILD_TESTS", "OFF")
        .define("LLAMA_BUILD_EXAMPLES", "OFF")
        .define("LLAMA_BUILD_SERVER", "OFF");

    // Handle MSVC specific flags for exception handling
    let target = env::var("TARGET").unwrap();
    if target.contains("msvc") {
        config.cxxflag("/EHsc");
    }

    // Build the project
    let dst = config.build();

    // 4. Linkage Configuration
    // Experts handle cases where libraries might be in 'lib' or 'lib64'
    let lib_dir = dst.join("lib");
    let lib64_dir = dst.join("lib64");
    if lib64_dir.exists() {
        println!("cargo:rustc-link-search=native={}", lib64_dir.display());
    } else {
        println!("cargo:rustc-link-search=native={}", lib_dir.display());
    }
    
    println!("cargo:rustc-link-lib=static=llama");
    
    // Link appropriate C++ standard library based on OS
    if target.contains("apple") {
        println!("cargo:rustc-link-lib=dylib=c++");
    } else if target.contains("linux") {
        println!("cargo:rustc-link-lib=dylib=stdc++");
    }
    // Windows MSVC links the CRT automatically based on the 'cmake' crate settings

    // 5. Binding Generation
    let bindings = bindgen::Builder::default()
        .header(header_path.to_str().expect("Valid UTF-8 path for header"))
        // Required for Rust Edition 2024: generates 'unsafe extern "C"'
        .rust_edition(bindgen::RustEdition::Edition2024)
        // Inform bindgen of all necessary include directories
        .clang_arg(format!("-I{}", include_path.display()))
        .clang_arg(format!("-I{}", ggml_include_path.display()))
        // Use modern cargo callbacks for better change detection
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Optimization: Only generate what's strictly necessary for the public API
        .allowlist_function("llama_.*")
        .allowlist_type("llama_.*")
        .allowlist_var("llama_.*")
        // Portable C types (e.g., use c_int instead of i32 where appropriate)
        .ctypes_prefix("core::ffi")
        .generate()
        .expect("Failed to generate Rust bindings for llama.cpp");

    // 6. Output to File
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings to OUT_DIR");
}