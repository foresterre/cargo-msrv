#[derive(Debug, Default)]
pub struct CargoCommand {
    features: Option<Vec<String>>,
    all_features: bool,
    no_default_features: bool,
    target: Option<String>,
}

impl CargoCommand {
    /// Set the features to be forwarded as `cargo <cmd> --features`
    pub fn features(mut self, features: Option<Vec<String>>) -> Self {
        self.features = features;
        self
    }

    /// Set the `all features` flag to be forwarded as `cargo <cmd> --all-features`
    pub fn all_features(mut self, value: bool) -> Self {
        self.all_features = value;
        self
    }

    /// Set the `no default features` flag to be forwarded as `cargo <cmd> --no-default-features`
    pub fn no_default_features(mut self, value: bool) -> Self {
        self.no_default_features = value;
        self
    }

    /// Set the target flag to be forwarded as `cargo <cmd> --target
    pub fn target(mut self, target: Option<impl ToString>) -> Self {
        self.target = target.map(|t| t.to_string());
        self
    }

    /// Intended to be used in conjunction with [`RunCommand`] and/or [`RustupCommand`].
    ///
    /// [`RunCommand`]: crate::check::RunCommand
    /// [`RustupCommand`]: crate::external_command::rustup_command::RustupCommand
    // Currently we don't invoke it from here directly, but we might eventually, if
    // we want to also provide some nicer structs around parsing. However compared to
    // some other cargo subcommand crates, we also (currently) need rustup, so the invocation
    // would need to supply everything we supply to rustup.
    pub fn into_args(self) -> Vec<String> {
        // Eventually we should also add support for CARGO env var
        let mut args = Vec::<String>::with_capacity(8);

        // Currently only `cargo check` is used by cargo msrv.
        // Alternatives can be set when using cargo msrv -- custom cmd
        // This value does open the path to use cargo build for Rust < 1.16
        args.extend_from_slice(&["cargo".to_string(), "check".to_string()]);

        if let Some(features) = self.features {
            let features = features.join(",");

            args.extend_from_slice(&["--features".to_string(), features]);
        }

        // probably unnecessary to supply both this and --features, if both have a value, but
        // by adding both to the command separately, we can optimally invoke cargo's own behaviour
        if self.all_features {
            args.push("--all-features".to_string());
        }

        if self.no_default_features {
            args.push("--no-default-features".to_string());
        }

        if let Some(target) = self.target {
            args.push("--target".to_string());
            args.push(target);
        }

        args
    }
}

#[cfg(test)]
mod tests {
    use crate::external_command::cargo_command::CargoCommand;

    #[test]
    fn set_features_none() {
        let cargo_command = CargoCommand::default();
        let cargo_command = cargo_command.features(None);
        assert_eq!(
            cargo_command.into_args().join(" "),
            "cargo check".to_string()
        );
    }

    #[test]
    fn set_features_one() {
        let cargo_command = CargoCommand::default();
        let cargo_command = cargo_command.features(Some(vec!["pika".to_string()]));
        assert_eq!(
            cargo_command.into_args().join(" "),
            "cargo check --features pika".to_string()
        );
    }

    #[test]
    fn set_features_two() {
        let cargo_command = CargoCommand::default();
        let cargo_command =
            cargo_command.features(Some(vec!["chu".to_string(), "chris".to_string()]));
        assert_eq!(
            cargo_command.into_args().join(" "),
            "cargo check --features chu,chris".to_string()
        );
    }

    #[test]
    fn set_no_default_features() {
        let cargo_command = CargoCommand::default();
        let cargo_command = cargo_command.no_default_features(true);
        assert_eq!(
            cargo_command.into_args().join(" "),
            "cargo check --no-default-features".to_string()
        );
    }

    #[test]
    fn set_all_features() {
        let cargo_command = CargoCommand::default();
        let cargo_command = cargo_command.all_features(true);
        assert_eq!(
            cargo_command.into_args().join(" "),
            "cargo check --all-features".to_string()
        );
    }

    #[test]
    fn set_target_none() {
        let cargo_command = CargoCommand::default();
        let cargo_command = cargo_command.target(None::<String>);
        assert_eq!(
            cargo_command.into_args().join(" "),
            "cargo check".to_string()
        );
    }

    #[test]
    fn set_target_some() {
        let cargo_command = CargoCommand::default();
        let cargo_command = cargo_command.target(Some("some"));
        assert_eq!(
            cargo_command.into_args().join(" "),
            "cargo check --target some".to_string()
        );
    }

    #[test]
    fn combination_of_everything() {
        let cargo_command = CargoCommand::default();
        let cargo_command = cargo_command
            .features(Some(vec!["pika".to_string(), "chu".to_string()]))
            .all_features(true)
            .no_default_features(true)
            .target(Some("pickme"));

        let cmd = cargo_command.into_args().join(" ");
        assert!(cmd.contains("--all-features"));
        assert!(cmd.contains("--features pika,chu"));
        assert!(cmd.contains("--no-default-features"));
        assert!(cmd.contains("--target pickme"));
    }
}
