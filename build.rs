use std::env;
use std::path::PathBuf;

fn canonicalize(path: &std::path::Path) -> String {
    dunce::canonicalize(path)
        .unwrap_or_else(|_| path.to_path_buf())
        .to_str()
        .expect("Invalid UTF-8 path")
        .to_string()
}

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let nrd_include = canonicalize(&manifest_dir.join("NRD/Include"));
    let nri_include = canonicalize(&manifest_dir.join("NRI/Include"));
    let nrd_integration_include = canonicalize(&manifest_dir.join("NRD/Integration"));
    let src_dir = canonicalize(&manifest_dir.join("src"));

    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=src/wrapper.hpp");
    println!("cargo:rerun-if-changed=src/wrapper.cpp");
    println!("cargo:rerun-if-changed=src/CMakeLists.txt");
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_NRD_DEBUG_LOGGING");
    println!("cargo:rerun-if-env-changed=PROFILE");

    let debug_logging = env::var("CARGO_FEATURE_NRD_DEBUG_LOGGING").is_ok();

    let nrd_lib_dir = build_nrd();

    // autocxx passes include dirs as -I to clang via Builder::new.
    let mut autocxx_builder = autocxx_build::Builder::new(
        "src/lib.rs",
        &[
            &nrd_include,
            &nri_include,
            &nrd_integration_include,
            &src_dir,
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
        .include(&nrd_include)
        .compile("nrd-autocxx");

    let mut wrapper_build = cc::Build::new();
    wrapper_build
        .cpp(true)
        .flag_if_supported("-std=c++17")
        .flag_if_supported("/std:c++17")
        .include(&nrd_include)
        .include(&nri_include)
        .include(&nrd_integration_include)
        .file(manifest_dir.join("src/wrapper.cpp"));
    if debug_logging {
        wrapper_build.define("NRD_INTEGRATION_DEBUG_LOGGING", Some("1"));
    }
    wrapper_build.compile("nrd_wrapper");

    println!("cargo:rustc-link-search=native={}", nrd_lib_dir.display());
    println!("cargo:rustc-link-lib=static=NRD");
    println!("cargo:rustc-link-lib=static=NRI");
}

fn build_nrd() -> PathBuf {
    // Use cargo profile so cmake provides appropriate debug symbols / optimisation level.
    // CRT is forced to /MD by CMakeLists.txt to match Rust on MSVC.
    let profile = env::var("PROFILE").unwrap_or_default();
    let cmake_profile = if profile == "release" {
        "Release"
    } else {
        "Debug"
    };
    let dst = cmake::Config::new("src").profile(&cmake_profile).build();

    let search_subs = [
        "lib",
        "lib64",
        &format!("lib/{cmake_profile}"),
        &format!("lib64/{cmake_profile}"),
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

    // NRI sub-libraries are not installed to lib/; search build/NRI/ instead.
    let nri_build_dir = dst.join("build").join("NRI");
    let nri_build_config_dir = nri_build_dir.join(&cmake_profile);
    if nri_build_dir.exists() {
        println!("cargo:rustc-link-search=native={}", nri_build_dir.display());
    }
    if nri_build_config_dir.exists() {
        println!("cargo:rustc-link-search=native={}", nri_build_config_dir.display());
    }
    for &lib in &["NRI_Shared", "NRI_VK", "NRI_Validation"] {
        println!("cargo:rustc-link-lib=static={}", lib);
    }

    // ShaderMakeBlob is pulled in by cmake's FetchContent; path is predictable.
    let smb_dir = dst.join("build/_deps/shadermake-build").join(&cmake_profile);
    let smb_files = [format!("libShaderMakeBlob.a"), format!("ShaderMakeBlob.lib")];
    if smb_files.iter().any(|f| smb_dir.join(f).exists()) {
        println!("cargo:rustc-link-search=native={}", smb_dir.display());
        println!("cargo:rustc-link-lib=static=ShaderMakeBlob");
    }

    nrd_lib_dir
}
