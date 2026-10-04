use std::path::PathBuf;

use asset_transport::{ensure_artifacts_dir, games_root_from_env};

#[cfg(windows)]
mod first_run;

#[global_allocator]
static PROCESS_ALLOCATOR: diag::ProcessCountingAllocator = diag::ProcessCountingAllocator;

fn main() {
    bootstrap::bench::arm();
    prepare_process_root().unwrap_or_else(|e| {
        diag::exit_launch_error(&e);
    });
    #[cfg_attr(not(windows), allow(unused_mut))]
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().is_some_and(|arg| arg == "server") {
        let bind = match args.as_slice() {
            [_] => "127.0.0.1:28980".parse(),
            [_, flag, address] if flag == "--bind" => address.parse(),
            _ => diag::exit_launch_error("Usage: iw4l server [--bind IP:PORT]"),
        }
        .unwrap_or_else(|_| diag::exit_launch_error("The server bind address must be IP:PORT"));
        bootstrap::standalone::run_server(bind)
            .unwrap_or_else(|error| diag::exit_launch_error(&error));
        return;
    }
    if args.first().is_some_and(|arg| arg == "join") {
        let address = match args.as_slice() {
            [_, address] => address.parse(),
            _ => diag::exit_launch_error("Usage: iw4l join IP:PORT"),
        }
        .unwrap_or_else(|_| diag::exit_launch_error("The server address must be IP:PORT"));
        bootstrap::run_connected(address).unwrap_or_else(|error| diag::exit_launch_error(&error));
        return;
    }
    if args.is_empty() || args.first().is_some_and(|arg| arg == "game") {
        if args.len() > 1 {
            diag::exit_launch_error("The game command accepts no additional arguments");
        }
        bootstrap::run_native().unwrap_or_else(|e| diag::exit_launch_error(&e));
        return;
    }
    #[cfg(windows)]
    {
        // Also for a shortcut that names a map, so it works on first launch.
        first_run::prepare().unwrap_or_else(|e| first_run::fail(&e));
        if args.is_empty() {
            args.push("menu".into());
        }
    }
    let artifacts = ensure_artifacts_dir().unwrap_or_else(|e| diag::exit_launch_error(&e));
    announce_log(diag::init_log(&artifacts));
    let (mode, acceptance) =
        bootstrap::parse_cli(args.into_iter()).unwrap_or_else(|e| diag::exit_launch_error(&e));
    let games = games_root_from_env().unwrap_or_else(|e| diag::exit_launch_error(&e));
    bootstrap::launch(games, artifacts, mode, acceptance);
}

fn prepare_process_root() -> Result<(), String> {
    #[cfg(windows)]
    {
        let exe =
            std::env::current_exe().map_err(|error| format!("cannot locate iw4l.exe: {error}"))?;
        let root = exe
            .parent()
            .ok_or_else(|| format!("iw4l.exe has no parent directory: {}", exe.display()))?;
        std::env::set_current_dir(root).map_err(|error| {
            format!(
                "cannot enter launcher directory {}: {error}",
                root.display()
            )
        })?;
    }
    Ok(())
}

fn announce_log(path: PathBuf) {
    diag::announce_log_stdout(&path, diag::latest_log_path().as_deref());
    diag::info!(Launch, "log: {}", path.display());
}
