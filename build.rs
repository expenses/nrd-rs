use std::env;
use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let nrd_include = manifest_dir
        .join("NRD/Include")
        .canonicalize()
        .expect("Failed to locate NRD include directory");
    let nrd_include_str = nrd_include
        .to_str()
        .expect("Invalid UTF-8 path")
        .to_string();

    let nri_include = manifest_dir
        .join("NRI/Include")
        .canonicalize()
        .expect("Failed to locate NRI include directory");
    let nri_include_str = nri_include
        .to_str()
        .expect("Invalid UTF-8 path")
        .to_string();

    let nrd_integration_include = manifest_dir
        .join("NRD/Integration")
        .canonicalize()
        .expect("Failed to locate NRD Integration include directory");
    let nrd_integration_include_str = nrd_integration_include
        .to_str()
        .expect("Invalid UTF-8 path")
        .to_string();

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let src_dir = manifest_dir.join("src");
    let src_dir_str = src_dir.to_str().expect("Invalid UTF-8 path").to_string();

    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=src/wrapper.hpp");
    println!("cargo:rerun-if-changed=src/wrapper.cpp");
    println!("cargo:rerun-if-changed=src/CMakeLists.txt");
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_NRD_DEBUG_LOGGING");

    let debug_logging = env::var("CARGO_FEATURE_NRD_DEBUG_LOGGING").is_ok();

    // Build NRD + NRI static libraries via cmake
    let nrd_lib_dir = build_nrd();

    // Generate autocxx/cxx bindings from wrapper.hpp (includes NRD types)
    let mut autocxx_builder = autocxx_build::Builder::new(
        "src/lib.rs",
        &[
            &nrd_include_str,
            nri_include.to_str().unwrap(),
            &nrd_integration_include.to_str().unwrap(),
            &src_dir_str,
        ],
    );
    if debug_logging {
        autocxx_builder = autocxx_builder.extra_clang_args(&["-DNRD_INTEGRATION_DEBUG_LOGGING"]);
    }
    let mut cc_build = autocxx_builder
        .build()
        .expect("autocxx code generation failed");
    if debug_logging {
        cc_build.define("NRD_INTEGRATION_DEBUG_LOGGING", Some("1"));
    }
    cc_build
        .flag_if_supported("-std=c++17")
        .include(&nrd_include_str)
        .compile("nrd-autocxx");

    // Compile wrapper.cpp with access to NRI + NRDIntegration
    let mut wrapper_build = cc::Build::new();
    wrapper_build
        .cpp(true)
        .flag_if_supported("-std=c++17")
        .flag_if_supported("/std:c++17")
        .include(&nrd_include_str)
        .include(&nri_include_str)
        .include(&nrd_integration_include_str)
        .file(manifest_dir.join("src/wrapper.cpp"));
    if debug_logging {
        wrapper_build.define("NRD_INTEGRATION_DEBUG_LOGGING", Some("1"));
    }
    wrapper_build.compile("nrd_wrapper");

    // Link NRD, NRI, and their dependencies
    println!("cargo:rustc-link-search=native={}", nrd_lib_dir.display());
    println!("cargo:rustc-link-lib=static=NRD");
    for &lib in &["NRI", "NRI_Shared", "NRI_VK"] {
        let candidate = nrd_lib_dir.join(format!("lib{}.a", lib));
        if candidate.exists() {
            println!("cargo:rustc-link-lib=static={}", lib);
        }
    }

    // Search for ShaderMakeBlob (cmake FetchContent location varies by version)
    if let Some(dir) = find_lib(&out_dir.join("build"), "libShaderMakeBlob.a") {
        println!("cargo:rustc-link-search=native={}", dir.display());
        println!("cargo:rustc-link-lib=static=ShaderMakeBlob");
    }
}

fn find_lib(dir: &std::path::Path, target: &str) -> Option<PathBuf> {
    if dir.join(target).exists() {
        return Some(dir.to_path_buf());
    }
    for entry in std::fs::read_dir(dir).ok()? {
        let entry = entry.ok()?;
        let path = entry.path();
        if path.is_dir() {
            if let Some(found) = find_lib(&path, target) {
                return Some(found);
            }
        }
    }
    None
}

fn build_nrd() -> PathBuf {
    let dst = cmake::Config::new("src").build();
    let profile = env::var("PROFILE").unwrap_or_else(|_| "Debug".to_string());
    let cap_profile = if profile == "debug" {
        "Debug"
    } else {
        "Release"
    };

    let search_subs = [
        "lib",
        "lib64",
        &format!("lib/{}", cap_profile),
        &format!("lib64/{}", cap_profile),
    ];

    let mut nrd_lib_dir = None;
    for sub in &search_subs {
        let candidate = dst.join(sub);
        if candidate.join("libNRD.a").exists() || candidate.join("NRD.lib").exists() {
            nrd_lib_dir = Some(candidate);
            break;
        }
    }
    let nrd_lib_dir = nrd_lib_dir.unwrap_or_else(|| {
        panic!(
            "libNRD.a / NRD.lib not found under target directory {:?}",
            dst
        )
    });

    // NRI sub-libraries (NRI_Shared, NRI_VK, NRI_Validation) may not be
    // installed to lib/; check build/NRI/ as a fallback.
    let nri_build_dir = dst.join("build").join("NRI");
    if nri_build_dir.exists() {
        println!("cargo:rustc-link-search=native={}", nri_build_dir.display());
    }
    for &lib in &["NRI_Shared", "NRI_VK", "NRI_Validation"] {
        let candidate = nrd_lib_dir.join(format!("lib{}.a", lib));
        let candidate2 = nri_build_dir.join(format!("lib{}.a", lib));
        if candidate.exists() || candidate2.exists() {
            println!("cargo:rustc-link-lib=static={}", lib);
        }
    }

    nrd_lib_dir
}
