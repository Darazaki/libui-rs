extern crate cmake;

use cmake::Config;

use std::env;
use std::path::Path;
use std::process::Command;

fn target_os() -> String {
    env::var("CARGO_CFG_TARGET_OS").unwrap_or(String::new())
}

fn main() {
    // Fetch the submodule if needed
    if cfg!(feature = "fetch") {
        // Init or update the submodule with libui if needed
        if !Path::new("libui/.git").exists() {
            Command::new("git")
                .args(&["version"])
                .status()
                .expect("Git does not appear to be installed. Error");
            Command::new("git")
                .args(&["submodule", "update", "--init"])
                .status()
                .expect("Unable to init libui submodule. Error");
        } else {
            Command::new("git")
                .args(&["submodule", "update", "--recursive"])
                .status()
                .expect("Unable to update libui submodule. Error");
        }
    }

    // Deterimine if we're building for MSVC
    let target = env::var("TARGET").unwrap();
    let msvc = target.contains("msvc");
    // Build libui if needed. Otherwise, assume it's in lib/
    let mut dst;
    if cfg!(feature = "build") {
        let mut cfg = Config::new("libui");
        cfg.build_target("libui").profile("release");

        if cfg!(feature = "static") {
            cfg.define("BUILD_SHARED_LIBS", "OFF");
            // When cross compiling, clang/gcc (correctly) errors on narrowing not allowed in c++11.
            // Disable that until libui fixes that.
            if target_os() == "windows" {
                //cfg.cxxflag("-Wno-c++11-narrowing");
            }
        }

        dst = cfg.build();

        let mut postfix = Path::new("build").join("out");
        if msvc {
            postfix = postfix.join("Release");
        }
        dst = dst.join(&postfix);
    } else {
        dst = env::current_dir().expect("Unable to retrieve current directory location.");
        dst.push("lib");
    }

    let libname = if msvc { "libui" } else { "ui" };

    println!("cargo:rustc-link-search=native={}", dst.display());
    
    if cfg!(feature = "static") {
        println!("cargo:rustc-link-lib=static={}", libname);
    } else {
        println!("cargo:rustc-link-lib={}", libname);
    }

    if cfg!(feature = "static") && target_os() == "linux" {
        let out = Command::new("pkg-config")
            .args(&["--libs", "gtk+-3.0"])
            .output()
            .expect("pkg-config does not appear to be installed.");

        if !out.status.success() {
            panic!("couldn't find gtk+-3.0");
        }

        for lib in std::str::from_utf8(&out.stdout)
            .expect("invalid output from pkg-config.")
            .split(' ')
        {
            if lib.len() > 2 {
                assert!(&lib[..2] == "-l");
                println!("cargo:rustc-link-lib={}", &lib[2..]);
            }
        }
    } else if cfg!(feature = "static") && target_os() == "windows" {
        for lib in &[
            "user32", "gdi32", "comctl32", "ole32", "d2d1", "uxtheme", "dwrite",
        ] {
            println!("cargo:rustc-link-lib={}", lib);
        }
    }

    if cfg!(all(not(target_os = "windows"), feature = "static")) {
        if let Some(cmd) = match env::var("TARGET").unwrap().as_str() {
            "x86_64-pc-windows-gnu" => Some("x86_64-w64-mingw32-windres"),
            "i686-pc-windows-gnu" => Some("i686-w64-mingw32-windres"),
            _ => None,
        } {
            let prefix = "resource";
            let resource = "resource.rc";

            Command::new(cmd)
                .args(&[
                    "--input",
                    resource,
                    "--output-format=coff",
                    &format!("--output={}/lib{}.a", dst.display(), prefix),
                ])
                .status()
                .expect("could not run windres");

            println!("cargo:rustc-link-lib=static={}", prefix);
        }
    }
}
