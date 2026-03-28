use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    // 1. Path Management
    // Use CARGO_MANIFEST_DIR to anchor paths absolutely to avoid issues with relative path resolution
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let llama_cpp_root = manifest_dir
        .join("..")
        .join("..")
        .join("third_party")
        .join("llama.cpp");

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
        .profile("Release")
        .define("BUILD_SHARED_LIBS", "OFF")
        .define("LLAMA_BUILD_TESTS", "OFF")
        .define("LLAMA_BUILD_EXAMPLES", "OFF")
        .define("LLAMA_BUILD_SERVER", "OFF")
        .define("GGML_STATIC", "ON");

    // Handle MSVC specific flags for exception handling
    let target = env::var("TARGET").unwrap();
    if target.contains("msvc") {
        config.cxxflag("/EHsc");
        // ggml-cpu uses RegOpenKeyExA and RegQueryValueExA to query CPU flags on Windows
        println!("cargo:rustc-link-lib=dylib=advapi32");
    }

    // Build the project
    let dst = config.build();

    // 4. Linkage Configuration
    // MSVC generates multi-config outputs (e.g., lib/Release or build/Release)
    // Since we forced the 'Release' profile above, we know exactly what subfolder CMake used.
    let cmake_profile = "Release";

    let search_paths = vec![
        dst.join("lib"),
        dst.join("lib").join(cmake_profile),
        dst.join("lib64"),
        dst.join("lib64").join(cmake_profile),
        dst.join("build"),
        dst.join("build").join(cmake_profile),
        dst.join("build").join("common"),
    ];

    let mut found_libs = HashSet::new();

    for path in &search_paths {
        if path.exists() {
            // Inform Cargo about the library search path
            println!("cargo:rustc-link-search=native={}", path.display());

            // Automatically discover and link all static libraries (.a or .lib)
            if let Ok(entries) = fs::read_dir(path) {
                for entry in entries.flatten() {
                    let file_name = entry.file_name().to_string_lossy().to_string();

                    // Logic for Linux/macOS (.a) and Windows (.lib)
                    let lib_name = if file_name.starts_with("lib") && file_name.ends_with(".a") {
                        Some(file_name.trim_start_matches("lib").trim_end_matches(".a"))
                    } else if file_name.ends_with(".lib") {
                        Some(file_name.trim_end_matches(".lib"))
                    } else {
                        None
                    };

                    if let Some(name) = lib_name {
                        // Skip auxiliary libraries that are not core to llama.cpp
                        if !name.contains("test") && !name.contains("example") {
                            let name = name.to_string();
                            if found_libs.insert(name.clone()) {
                                println!("cargo:rustc-link-lib=static={}", name);
                            }
                        }
                    }
                }
            }
        }
    }

    // Fallback: Ensure core libraries are linked if discovery failed
    if !found_libs.contains("llama") {
        println!("cargo:rustc-link-lib=static=llama");
    }
    if !found_libs.contains("ggml") {
        println!("cargo:rustc-link-lib=static=ggml");
    }

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
