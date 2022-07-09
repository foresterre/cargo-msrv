//! An 'Action' is an activity which the program will perform to get from an input to an output.
//! Currently, all actions match one-on-one with subcommands, but this may change in the future.

#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    // Determines the MSRV for a project
    Find,
    // List the MSRV's as specified by package authors
    List,
    // Verifies the given MSRV
    Verify,
    // Set the MSRV in the Cargo manifest to a given value
    Set,
    // Shows the MSRV of the current crate as specified in the Cargo manifest
    Show,
}
