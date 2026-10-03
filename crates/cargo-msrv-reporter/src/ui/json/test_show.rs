use crate::JsonHandler;
use crate::event::ShowResult;
use camino::Utf8Path;
use cargo_msrv_types::BareVersion;
use storyteller::EventHandler;

#[test]
fn handler() {
    let event = ShowResult::new(
        BareVersion::ThreeComponents(1, 2, 3),
        Utf8Path::new("/hello/world").to_path_buf(),
    );

    let writer = Vec::new();
    let handler = JsonHandler::new(writer);
    handler.handle(event.into());

    let buffer = handler.inner_writer();
    let actual: serde_json::Value = serde_json::from_slice(buffer.as_slice()).unwrap();

    let expected = serde_json::json!({
        "type": "subcommand_result",
        "subcommand_id": "show",
        "result": {
            "version": "1.2.3",
            "manifest_path": "/hello/world"
        }
    });

    assert_eq!(actual, expected);
}

#[test]
fn event() {
    let event = ShowResult::new(
        BareVersion::ThreeComponents(1, 10, 100),
        Utf8Path::new("/hello/world").to_path_buf(),
    );

    let expected = serde_json::json!({
        "result": {
            "version": "1.10.100",
            "manifest_path": "/hello/world"
        }
    });

    let actual = serde_json::to_value(event).unwrap();
    assert_eq!(actual, expected);
}

#[test]
fn event_with_package() {
    let event = ShowResult::new(
        BareVersion::ThreeComponents(1, 10, 100),
        Utf8Path::new("/ws/a/Cargo.toml").to_path_buf(),
    )
    .with_package(Some(cargo_msrv_context::SelectedPackage {
        name: "a".to_string(),
        path: Utf8Path::new("/ws/a/Cargo.toml").to_path_buf(),
    }));

    let expected = serde_json::json!({
        "package": {
            "name": "a",
            "path": "/ws/a/Cargo.toml"
        },
        "result": {
            "version": "1.10.100",
            "manifest_path": "/ws/a/Cargo.toml"
        }
    });

    let actual = serde_json::to_value(event).unwrap();
    assert_eq!(actual, expected);
}
