use std::{
    env, fs,
    path::PathBuf,
    process::{Command, ExitCode},
};

fn main() -> ExitCode {
    let task = env::args().nth(1);
    match task.as_deref() {
        Some("build-wasm") => build_wasm(),
        _ => {
            eprintln!("Usage: cargo xtask <TASK>");
            eprintln!();
            eprintln!("Tasks:");
            eprintln!("  build-wasm   Build the wasm crate with wasm-pack (output -> web/pkg)");
            ExitCode::FAILURE
        }
    }
}

fn workspace_root() -> PathBuf {
    // CARGO_MANIFEST_DIR points to xtask/; go one level up.
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest.parent().expect("workspace root").to_owned()
}

fn build_wasm() -> ExitCode {
    let root = workspace_root();
    let wasm_dir = root.join("wasm");
    let out_dir = root.join("web").join("pkg");

    fs::create_dir_all(&out_dir).expect("create web/pkg");

    let out_dir_str = out_dir.to_string_lossy().into_owned();

    let status = Command::new("wasm-pack")
        .args(["build", "--target", "web", "--out-dir", &out_dir_str])
        .current_dir(&wasm_dir)
        .status()
        .expect("failed to run wasm-pack (is it installed?)");

    if status.success() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
