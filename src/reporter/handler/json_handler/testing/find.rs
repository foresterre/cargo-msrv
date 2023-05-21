use crate::context::SearchMethod;
use crate::manifest::bare_version::BareVersion;
use crate::reporter::event::FindResult;
use crate::reporter::JsonHandler;
use crate::semver;
use storyteller::EventHandler;

#[test]
fn compatible_handler() {
    let version = semver::Version::new(1, 2, 3);

    let event = FindResult::new_msrv(
        version,
        "x",
        BareVersion::TwoComponents(1, 0),
        BareVersion::TwoComponents(1, 10),
        SearchMethod::Linear,
    );

    let writer = Vec::new();
    let handler = JsonHandler::new(writer);
    handler.handle(event.into());

    let buffer = handler.inner_writer();
    let actual: serde_json::Value = serde_json::from_slice(buffer.as_slice()).unwrap();

    let expected = serde_json::json!({
        "type": "subcommand_result",
        "subcommand_id": "find",
        "result": {
            "success" : true,
            "version" : "1.2.3",
        },
    });

    assert_eq!(actual, expected);
}

#[test]
fn incompatible_handler() {
    let event = FindResult::none(
        "x",
        BareVersion::TwoComponents(1, 0),
        BareVersion::TwoComponents(1, 10),
        SearchMethod::Bisect,
    );

    let writer = Vec::new();
    let handler = JsonHandler::new(writer);
    handler.handle(event.into());

    let buffer = handler.inner_writer();
    let actual: serde_json::Value = serde_json::from_slice(buffer.as_slice()).unwrap();

    let expected = serde_json::json!({
        "type": "subcommand_result",
        "subcommand_id": "find",
        "result": {
            "success" : false,
        },
    });

    assert_eq!(actual, expected);
}

#[test]
fn compatible() {
    let version = semver::Version::new(1, 2, 3);
    let event = FindResult::new_msrv(
        version,
        "x",
        BareVersion::TwoComponents(1, 0),
        BareVersion::TwoComponents(1, 10),
        SearchMethod::Bisect,
    );

    let expected = serde_json::json!({
        "result": {
            "success" : true,
            "version" : "1.2.3",
        },
    });

    let actual = serde_json::to_value(event).unwrap();
    assert_eq!(actual, expected);
}

#[test]
fn incompatible() {
    let event = FindResult::none(
        "x",
        BareVersion::TwoComponents(1, 0),
        BareVersion::TwoComponents(1, 10),
        SearchMethod::Bisect,
    );

    let expected = serde_json::json!({
        "result": {
            "success" : false,
        },
    });

    let actual = serde_json::to_value(event).unwrap();
    assert_eq!(actual, expected);
}
