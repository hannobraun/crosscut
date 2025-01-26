use std::path::{Path, PathBuf};

use crosscut_game_engine::command::Command;
use crosscut_protocol::command::CommandExt;
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
};

use crate::{build_game::build_game_once, files::FILES};

pub async fn export(
    games_path: PathBuf,
    target_path: PathBuf,
) -> anyhow::Result<()> {
    let mut games = fs::read_dir(&games_path).await?;

    while let Some(game) = games.next_entry().await? {
        if !game.file_type().await?.is_dir() {
            continue;
        }

        let game_dir = game.path();
        let dir_within_games = game_dir.strip_prefix(&games_path)?;
        let target_path = target_path.join(dir_within_games);

        prepare_directory(&target_path).await?;
        export_static_files(&target_path).await?;
        export_game_code(&game_dir, &target_path).await?;
    }

    Ok(())
}

async fn prepare_directory(path: &Path) -> anyhow::Result<()> {
    // This is a bit too dangerous as-is. A small bug somewhere else could cause
    // this to delete the wrong directory.
    //
    // What should probably happen instead, is that when creating the directory,
    // we should add some special file that marks it as a Caterpillar export
    // directory. If that file isn't there, this code should refuse to do
    // anything.
    if path.exists() {
        fs::remove_dir_all(path).await?;
    }
    fs::create_dir_all(path).await?;

    Ok(())
}

async fn export_static_files(path: &Path) -> anyhow::Result<()> {
    let static_files = ["index.html", "crosscut_host.wasm"];

    for name in static_files {
        let file_path = PathBuf::from(name);
        let Some(file) = FILES.get(&file_path) else {
            unreachable!(
                "Only accessing files that were statically included into the \
                binary, but `{name}` is not available",
            );
        };

        File::create(path.join(file_path))
            .await?
            .write_all(file)
            .await?;
    }

    Ok(())
}

async fn export_game_code(
    game_dir: &Path,
    target_path: &Path,
) -> anyhow::Result<()> {
    let compiler_output = build_game_once(game_dir).await?;
    let command = Command::UpdateCode {
        instructions: compiler_output.instructions,
    }
    .serialize();

    let target_path = target_path.join("command-with-instructions");
    File::create(target_path).await?.write_all(&command).await?;

    Ok(())
}
