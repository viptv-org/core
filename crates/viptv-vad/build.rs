use std::{env, fs, path::Path};

fn sources(path: &Path, output: &mut Vec<std::path::PathBuf>) {
    for entry in fs::read_dir(path).expect("read vendored WebRTC directory") {
        let path = entry.expect("read vendored WebRTC entry").path();
        if path.is_dir() {
            sources(&path, output);
        } else if path.extension().is_some_and(|ext| ext == "c") {
            output.push(path);
        }
    }
}

fn main() {
    println!("cargo:rerun-if-changed=vendor");
    println!("cargo:rerun-if-changed=src/native.c");
    let target = env::var("CARGO_CFG_TARGET_OS").expect("target OS");
    assert_ne!(target, "unknown", "native VAD requires a native target");
    let mut files = Vec::new();
    sources(Path::new("vendor/webrtc/common_audio"), &mut files);
    files.sort();
    cc::Build::new()
        .files(files)
        .file("src/native.c")
        .include("vendor")
        .define("NDEBUG", None)
        .define(
            if target == "windows" {
                "WEBRTC_WIN"
            } else {
                "WEBRTC_POSIX"
            },
            None,
        )
        .flag_if_supported("-fvisibility=hidden")
        .warnings(false)
        .compile("viptv_webrtc_vad");
}
