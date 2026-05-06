#[cfg(feature = "static-build")]
fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    
    println!("cargo:rustc-link-search=native={}/src/autogen", manifest_dir);
    println!("cargo:rustc-link-lib=static=resid");
    println!("cargo:rustc-link-lib=stdc++");
    
    // Ensure we don't rebuild unless the static blob changes
    println!("cargo:rerun-if-changed=src/static_gen/libresid.a");
}

#[cfg(not(feature = "static-build"))]
fn main() -> miette::Result<()> {
	const USE_NEW_FILTER: bool = true;
	
    let mut src = vec![
        "src/resid10/dac.cc",
        "src/resid10/envelope.cc",
        "src/resid10/extfilt.cc",
        "src/resid10/pot.cc",
        "src/resid10/sid.cc",
        "src/resid10/version.cc",
        "src/resid10/voice.cc",
        "src/resid10/wave.cc",
        ];

    if USE_NEW_FILTER {
        src.push("src/resid10/filter8580new.cc");
    } else {
        src.push("src/resid10/filter.cc");
    }

    let path = std::path::PathBuf::from("src");
    autocxx_build::Builder::new("src/lib.rs", [&path]).build()?
        .define("VERSION", Some("\"1.0\""))
        .define("NEW_8580_FILTER", Some(if USE_NEW_FILTER {"1"} else {"0"}))
        .files(src)
        .flag_if_supported("-std=c++14")
        .flag_if_supported("-Wno-psabi")
        .warnings(false)
        .compile("resid");

    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=src/resid10/");
    Ok(())
}
