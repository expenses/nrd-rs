use std::env;
use std::path::{Path, PathBuf};

/// Returns a canonicalized UTF-8 absolute path, falling back to the raw path on error.
fn abs_path(path: &Path) -> String {
    dunce::canonicalize(path)
        .unwrap_or_else(|_| path.to_path_buf())
        .to_str()
        .expect("non-UTF-8 path")
        .to_string()
}

fn main() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    for f in [
        "src/lib.rs",
        "src/wrapper.hpp",
        "src/wrapper.cpp",
        "src/CMakeLists.txt",
    ] {
        println!("cargo:rerun-if-changed={f}");
    }

    let features: &[(&str, &str)] = &[
        ("CARGO_FEATURE_DEBUG_LOGGING", "NRD_INTEGRATION_DEBUG_LOGGING"),
        ("CARGO_FEATURE_VIEWPORT_OFFSET", "NRD_SUPPORTS_VIEWPORT_OFFSET"),
        ("CARGO_FEATURE_CHECKERBOARD", "NRD_SUPPORTS_CHECKERBOARD"),
        ("CARGO_FEATURE_HISTORY_CONFIDENCE", "NRD_SUPPORTS_HISTORY_CONFIDENCE"),
        ("CARGO_FEATURE_DISOCCLUSION_THRESHOLD_MIX", "NRD_SUPPORTS_DISOCCLUSION_THRESHOLD_MIX"),
        ("CARGO_FEATURE_ANTIFIREFLY", "NRD_SUPPORTS_ANTIFIREFLY"),
        ("CARGO_FEATURE_QUAD_INTRINSICS", "NRD_SUPPORTS_QUAD_INTRINSICS"),
    ];

    for &(env_var, _) in features {
        println!("cargo:rerun-if-env-changed={env_var}");
    }
    println!("cargo:rerun-if-env-changed=PROFILE");

    let all_states: Vec<(&str, bool)> = features
        .iter()
        .map(|&(env_var, define)| (define, env::var(env_var).is_ok()))
        .collect();

    // Only NRD library features (not integration) are passed to CMake.
    let nrd_states: Vec<(&str, bool)> = all_states
        .iter()
        .filter(|(define, _)| !define.starts_with("NRD_INTEGRATION_"))
        .copied()
        .collect();

    let nrd_lib_dir = build_nrd(&nrd_states);

    let nrd_include = abs_path(&manifest.join("NRD/Include"));
    let nri_include = abs_path(&manifest.join("NRI/Include"));
    let nrd_integration = abs_path(&manifest.join("NRD/Integration"));
    let src_dir = abs_path(&manifest.join("src"));

    // Flatten all define flags into clang -D arguments.
    let clang_defines: Vec<String> = all_states
        .iter()
        .map(|&(define, enabled)| format!("-D{}={}", define, if enabled { 1 } else { 0 }))
        .collect();
    let clang_refs: Vec<&str> = clang_defines.iter().map(|s| s.as_str()).collect();

    let mut autocxx = autocxx_build::Builder::new(
        "src/lib.rs",
        &[&nrd_include, &nri_include, &nrd_integration, &src_dir],
    );
    autocxx = autocxx.extra_clang_args(&clang_refs);

    let mut cc_build = autocxx.build().expect("autocxx code generation failed");
    for &(define, enabled) in &all_states {
        cc_build.define(define, if enabled { Some("1") } else { Some("0") });
    }
    cc_build
        .flag_if_supported("-std=c++17")
        .include(&nrd_include)
        .compile("nrd-autocxx");

    let mut wrapper = cc::Build::new();
    wrapper
        .cpp(true)
        .flag_if_supported("-std=c++17")
        .flag_if_supported("/std:c++17")
        .include(&nrd_include)
        .include(&nri_include)
        .include(&nrd_integration)
        .file(manifest.join("src/wrapper.cpp"));
    for &(define, enabled) in &all_states {
        wrapper.define(define, if enabled { Some("1") } else { Some("0") });
    }
    wrapper.compile("nrd_wrapper");

    println!("cargo:rustc-link-search=native={}", nrd_lib_dir.display());
    println!("cargo:rustc-link-lib=static=NRD");
    println!("cargo:rustc-link-lib=static=NRI");
}

fn build_nrd(nrd_states: &[(&str, bool)]) -> PathBuf {
    let profile = env::var("PROFILE").unwrap_or_default();
    let cmake_profile = if profile == "release" {
        "Release"
    } else {
        "Debug"
    };

    let mut config = cmake::Config::new("src");
    config.profile(cmake_profile);
    for &(define, enabled) in nrd_states {
        config.define(define, if enabled { "ON" } else { "OFF" });
    }
    let dst = config.build();

    // Installed NRD may land in lib/ or lib64/, with or without a config subdirectory.
    let nrd_lib_dir = ["lib", "lib64"]
        .into_iter()
        .flat_map(|base| [dst.join(base), dst.join(base).join(cmake_profile)])
        .find(|dir| dir.join("libNRD.a").exists() || dir.join("NRD.lib").exists())
        .unwrap_or_else(|| panic!("libNRD.a / NRD.lib not found under {dst:?}"));

    // NRI sub-libraries are not installed; link from the build tree.
    let nri_build = dst.join("build/NRI");
    let nri_build_config = nri_build.join(cmake_profile);
    for dir in [&nri_build, &nri_build_config] {
        if dir.exists() {
            println!("cargo:rustc-link-search=native={}", dir.display());
        }
    }
    for lib in ["NRI_Shared", "NRI_VK", "NRI_Validation"] {
        println!("cargo:rustc-link-lib=static={lib}");
    }

    // ShaderMakeBlob is pulled in via CMake FetchContent; its path is predictable.
    let smb_dir = dst.join("build/_deps/shadermake-build").join(cmake_profile);
    if ["libShaderMakeBlob.a", "ShaderMakeBlob.lib"]
        .iter()
        .any(|f| smb_dir.join(f).exists())
    {
        println!("cargo:rustc-link-search=native={}", smb_dir.display());
        println!("cargo:rustc-link-lib=static=ShaderMakeBlob");
    }

    nrd_lib_dir
}
