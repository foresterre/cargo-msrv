use crate::config::Config;
use crate::manifest::bare_version::BareVersion;
use crate::reporter::event::{SetResult, ShowResult, VerifyResult};
use crate::reporter::JsonHandler;
use crate::semver;
use crate::toolchain::OwnedToolchainSpec;
use crate::SubcommandId;
use std::ops::Deref;
use std::path::Path;
use storyteller::EventHandler;

#[test]
fn handler_success() {
    let event = VerifyResult::compatible(OwnedToolchainSpec::new(
        &semver::Version::new(1, 2, 3),
        "my-target",
    ));

    let writer = Vec::new();
    let handler = JsonHandler::new(writer);
    handler.handle(event.into());

    let buffer = handler.inner_writer();
    let actual: serde_json::Value = serde_json::from_slice(buffer.as_slice()).unwrap();

    let expected = serde_json::json!({
        "type": "subcommand_result",
        "subcommand_id": "verify",
        "result": {
            "toolchain": {
                "target": "my-target",
                "version": "1.2.3",
            },
            "is_compatible": true,
        }
    });

    assert_eq!(actual, expected);
}

#[test]
fn event_success() {
    let event = VerifyResult::compatible(OwnedToolchainSpec::new(
        &semver::Version::new(1, 2, 3),
        "my-target",
    ));

    let expected = serde_json::json!({
        "result": {
            "toolchain": {
                "target": "my-target",
                "version": "1.2.3",
            },
            "is_compatible": true,
        }
    });

    let actual = serde_json::to_value(event).unwrap();
    assert_eq!(actual, expected);
}

#[test]
fn handler_failure_with_message() {
    let event = VerifyResult::incompatible(
        OwnedToolchainSpec::new(&semver::Version::new(1, 2, 3), "my-target"),
        Some("Hello World".to_string()),
    );

    let writer = Vec::new();
    let handler = JsonHandler::new(writer);
    handler.handle(event.into());

    let buffer = handler.inner_writer();
    let actual: serde_json::Value = serde_json::from_slice(buffer.as_slice()).unwrap();

    let expected = serde_json::json!({
        "type": "subcommand_result",
        "subcommand_id": "verify",
        "result": {
            "toolchain": {
                "target": "my-target",
                "version": "1.2.3",
            },
            "is_compatible": false,
            "error": "Hello World",
        }
    });

    assert_eq!(actual, expected);
}

#[test]
fn event_failure_with_message() {
    let event = VerifyResult::incompatible(
        OwnedToolchainSpec::new(&semver::Version::new(1, 2, 3), "my-target"),
        Some("Hello World".to_string()),
    );

    let expected = serde_json::json!({
        "result": {
            "toolchain": {
                "target": "my-target",
                "version": "1.2.3",
            },
            "is_compatible": false,
            "error": "Hello World",
        }
    });

    let actual = serde_json::to_value(event).unwrap();
    assert_eq!(actual, expected);
}

#[test]
fn handler_failure_without_message() {
    let event = VerifyResult::incompatible(
        OwnedToolchainSpec::new(&semver::Version::new(1, 2, 3), "my-target"),
        None,
    );

    let writer = Vec::new();
    let handler = JsonHandler::new(writer);
    handler.handle(event.into());

    let buffer = handler.inner_writer();
    let actual: serde_json::Value = serde_json::from_slice(buffer.as_slice()).unwrap();

    let expected = serde_json::json!({
        "type": "subcommand_result",
        "subcommand_id": "verify",
        "result": {
            "toolchain": {
                "target": "my-target",
                "version": "1.2.3",
            },
            "is_compatible": false,
        }
    });

    assert_eq!(actual, expected);
}

#[test]
fn event_failure_without_message() {
    let event = VerifyResult::incompatible(
        OwnedToolchainSpec::new(&semver::Version::new(1, 2, 3), "my-target"),
        None,
    );

    let expected = serde_json::json!({
        "result": {
            "toolchain": {
                "target": "my-target",
                "version": "1.2.3",
            },
            "is_compatible": false,
        }
    });

    let actual = serde_json::to_value(event).unwrap();
    assert_eq!(actual, expected);
}
