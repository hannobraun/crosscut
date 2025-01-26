use std::{
    env,
    fs::{self, File},
    path::Path,
};

fn main() -> anyhow::Result<()> {
    if let Err(err) = env::var("FILES") {
        match err {
            env::VarError::NotPresent => {
                create_dummy_files()?;
            }
            env::VarError::NotUnicode(_) => {
                return Err(err.into());
            }
        }
    }

    println!("cargo::rerun-if-env-changed=FILES");

    Ok(())
}

fn create_dummy_files() -> anyhow::Result<()> {
    let out_dir_var = env::var("OUT_DIR")?;

    let out_dir_path = Path::new(&out_dir_var);
    let files_path = out_dir_path.join("files");

    fs::create_dir_all(&files_path)?;

    let files = [
        "crosscut-debugger_bg.wasm",
        "crosscut-debugger.js",
        "crosscut_host.wasm",
        "index.html",
        "index-debugger.html",
        "tailwind.js",
    ];

    for file in files {
        File::create(files_path.join(file))?;
    }

    println!("cargo:rustc-env=FILES={}", files_path.display());

    Ok(())
}
