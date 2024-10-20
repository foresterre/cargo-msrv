use crate::manifest::bare_version::BareVersion;
use crate::reporter::event::SetResult;
use crate::reporter::JsonHandler;
use camino::Utf8Path;
use storyteller::EventHandler;

#[test]
fn handler() {
    let event = SetResult::new(
        BareVersion::TwoComponents(1, 10),
        Utf8Path::new("/hello/world").to_path_buf(),
    );

    let writer = Vec::new();
    let handler = JsonHandler::new(writer);

    handler.handle(event.into());

    let buffer = handler.inner_writer();
    let actual: serde_json::Value = serde_json::from_slice(buffer.as_slice()).unwrap();

    let expected = serde_json::json!({
        "type": "subcommand_result",
        "subcommand_id": "set",
        "result": {
            "version": "1.10",
            "manifest_path": "/hello/world"
        }
    });

    assert_eq!(actual, expected);
}

#[test]
fn event() {
    let event = SetResult::new(
        BareVersion::TwoComponents(1, 10),
        Utf8Path::new("/hello/world").to_path_buf(),
    );

    let expected = serde_json::json!({
        "result": {
            "version": "1.10",
            "manifest_path": "/hello/world"
        }
    });

    let actual = serde_json::to_value(event).unwrap();
    assert_eq!(actual, expected);
}
