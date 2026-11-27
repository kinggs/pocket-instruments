// Links the core's static library (D123 item 2): the one `make build` writes and the C++ harness
// and Rack link; and since issue 140 the beat file's, beside it. PI_CORE_LIB names it (the Makefile passes the library `make test` just checked);
// without it, this builds one from ../core with the command `make build` uses.
//
// SPDX-License-Identifier: MIT
use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-env-changed=PI_CORE_LIB");
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let root = manifest.parent().unwrap().to_path_buf();
    let lib = match std::env::var("PI_CORE_LIB") {
        Ok(p) if !p.is_empty() => PathBuf::from(p),
        _ => {
            let out = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("core");
            let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
            let status = Command::new(cargo)
                .current_dir(&root)
                .args(["rustc", "-p", "pi-core", "--release", "--locked", "--crate-type", "staticlib", "--target-dir"])
                .arg(&out)
                // A build script runs with the host's flags set for this crate; the core takes none.
                .env_remove("RUSTFLAGS")
                .env_remove("CARGO_ENCODED_RUSTFLAGS")
                .status()
                .expect("could not run cargo to build the core");
            assert!(status.success(), "building the core's static library failed");
            for src in ["core/src", "core/Cargo.toml", "core/include/pi_engine.h"] {
                println!("cargo:rerun-if-changed={}", root.join(src).display());
            }
            out.join("release").join("libpi_core.a")
        }
    };
    assert!(lib.is_file(), "no core library at {}", lib.display());
    println!("cargo:rerun-if-changed={}", lib.display());
    println!("cargo:rustc-link-search=native={}", lib.parent().unwrap().display());
    println!("cargo:rustc-link-lib=static=pi_core");
    // The beat file's reader and writer (docs/BEAT.md, beat/include/pi_beat.h; issue 140): its own
    // static library beside the core's. PI_BEAT_LIB names it; without it, built as `make build` does.
    println!("cargo:rerun-if-env-changed=PI_BEAT_LIB");
    let beat = match std::env::var("PI_BEAT_LIB") {
        Ok(p) if !p.is_empty() => PathBuf::from(p),
        _ => {
            let out = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("beat");
            let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
            let status = Command::new(cargo)
                .current_dir(&root)
                .args(["rustc", "-p", "pi-beat", "--release", "--locked", "--crate-type", "staticlib", "--target-dir"])
                .arg(&out)
                .env_remove("RUSTFLAGS")
                .env_remove("CARGO_ENCODED_RUSTFLAGS")
                .status()
                .expect("could not run cargo to build the beat library");
            assert!(status.success(), "building the beat file's static library failed");
            for src in ["beat/src", "beat/Cargo.toml", "core/src/snapshot_layout.rs"] {
                println!("cargo:rerun-if-changed={}", root.join(src).display());
            }
            out.join("release").join("libpi_beat.a")
        }
    };
    assert!(beat.is_file(), "no beat library at {}", beat.display());
    println!("cargo:rerun-if-changed={}", beat.display());
    if beat.parent() != lib.parent() {
        println!("cargo:rustc-link-search=native={}", beat.parent().unwrap().display());
    }
    println!("cargo:rustc-link-lib=static=pi_beat");
    // The commit, for --version. Absent outside a git checkout (a source tarball): "unknown".
    let commit = Command::new("git")
        .args(["-C", root.to_str().unwrap(), "rev-parse", "--short=7", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|| "unknown".into());
    println!("cargo:rustc-env=PI_COMMIT={commit}");
    println!("cargo:rerun-if-changed={}", root.join(".git/HEAD").display());
}
