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

    for f in ["src/lib.rs", "src/wrapper.hpp", "src/wrapper.cpp", "src/CMakeLists.txt"] {
        println!("cargo:rerun-if-changed={f}");
    }
    for v in ["CARGO_FEATURE_NRD_DEBUG_LOGGING", "PROFILE"] {
        println!("cargo:rerun-if-env-changed={v}");
    }

    let debug_logging = env::var("CARGO_FEATURE_NRD_DEBUG_LOGGING").is_ok();
    let nrd_lib_dir = build_nrd();

    let nrd_include     = abs_path(&manifest.join("NRD/Include"));
    let nri_include     = abs_path(&manifest.join("NRI/Include"));
    let nrd_integration = abs_path(&manifest.join("NRD/Integration"));
    let src_dir         = abs_path(&manifest.join("src"));

    let mut autocxx = autocxx_build::Builder::new(
        "src/lib.rs",
        &[&nrd_include, &nri_include, &nrd_integration, &src_dir],
    );
    if debug_logging {
        autocxx = autocxx.extra_clang_args(&["-DNRD_INTEGRATION_DEBUG_LOGGING"]);
    }
    let mut cc_build = autocxx.build().expect("autocxx code generation failed");
    if debug_logging {
        cc_build.define("NRD_INTEGRATION_DEBUG_LOGGING", Some("1"));
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
    if debug_logging {
        wrapper.define("NRD_INTEGRATION_DEBUG_LOGGING", Some("1"));
    }
    wrapper.compile("nrd_wrapper");

    println!("cargo:rustc-link-search=native={}", nrd_lib_dir.display());
    println!("cargo:rustc-link-lib=static=NRD");
    println!("cargo:rustc-link-lib=static=NRI");
}

fn build_nrd() -> PathBuf {
    let profile = env::var("PROFILE").unwrap_or_default();
    let cmake_profile = if profile == "release" { "Release" } else { "Debug" };

    let dst = cmake::Config::new("src").profile(cmake_profile).build();

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
    if ["libShaderMakeBlob.a", "ShaderMakeBlob.lib"].iter().any(|f| smb_dir.join(f).exists()) {
        println!("cargo:rustc-link-search=native={}", smb_dir.display());
        println!("cargo:rustc-link-lib=static=ShaderMakeBlob");
    }

    nrd_lib_dir
}