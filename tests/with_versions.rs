extern crate cargo_msrv;

use cargo_msrv::fetch::RustStableVersion;
use indicatif::ProgressBar;
use std::path::PathBuf;

#[test]
fn versions() {
    let features_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("features");

    for path in std::fs::read_dir(features_path).unwrap() {
        let path = path.unwrap();
        let meta = &path.metadata().unwrap();
        if meta.is_dir() {
            let project_dir = path.path();
            println!("dir: {:?}", &project_dir);
            let arguments = with_args(project_dir.clone())
                .into_iter()
                .collect::<Vec<String>>();

            let matches = cargo_msrv::cli::cli().get_matches_from(arguments);
            let matches = cargo_msrv::cli::cmd_matches(&matches).unwrap();

            println!("matches: {:?}", &matches);

            let result = cargo_msrv::msrv(
                &matches,
                RustStableVersion::new(1, 38, 0),
                &ProgressBar::new_spinner(),
            )
            .unwrap();

            println!("result: {:?}", &result);

            let expected = project_dir.clone();
            let expected = expected.iter().last().unwrap();
            let expected = expected.to_str().unwrap();
            let expected =
                RustStableVersion::from_parts(&expected.split('.').collect::<Vec<_>>()).unwrap();

            assert_eq!(result.unwrap(), expected);
        }
    }
}

fn with_args(path: PathBuf) -> impl IntoIterator<Item = String> {
    let path = path.as_os_str().to_string_lossy().to_string();

    let args = format!("cargo msrv --path {}", path);
    let args = args
        .split_ascii_whitespace()
        .map(ToString::to_string)
        .collect::<Vec<String>>();

    args
}
