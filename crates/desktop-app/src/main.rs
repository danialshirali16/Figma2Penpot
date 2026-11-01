fn main() -> eframe::Result {
    let paths = std::env::args_os()
        .skip(1)
        .map(std::path::PathBuf::from)
        .collect();
    desktop_app::run_with_paths(paths)
}
