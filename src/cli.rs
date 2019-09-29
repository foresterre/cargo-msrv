use clap::{App, AppSettings, Arg, SubCommand};

pub fn cli() -> App<'static, 'static> {
    App::new(env!("CARGO_PKG_NAME"))
        .bin_name("cargo")
        .usage("cargo msrv <SUBCOMMANDS>")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Martijn Gribnau <garm@ilumeo.com>")
        .global_setting(AppSettings::NextLineHelp)
        .global_setting(AppSettings::ColoredHelp)
        .global_setting(AppSettings::ColorAuto)
        .global_setting(AppSettings::DontCollapseArgsInUsage)
        .global_setting(AppSettings::UnifiedHelpMessage)
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .max_term_width(120)
        .subcommand(
            SubCommand::with_name("msrv").subcommand(
                SubCommand::with_name("seek")
                    .arg(
                        Arg::with_name("channel")
                            .long("channel")
                            .help("(WIP) part of target triple")
                            .possible_values(&["stable", "beta", "nightly", "master"])
                            .default_value("stable")
                            .takes_value(true),
                    )
                    //                    .arg(
                    //                        Arg::with_name("host")
                    //                            .long("host")
                    //                            .help("(WIP) part of target triple")
                    //                            .takes_value(true),
                    //                    )
                    .arg(
                        Arg::with_name("strategy")
                            .long("strategy")
                            .help("(WIP) The method used to determine the version")
                            .possible_values(&["new-to-old"])
                            .takes_value(true),
                    ), //                    .arg(
                       //                        Arg::with_name("ignore-toolchain-file")
                       //                            .long("ignore-toolchain-file")
                       //                            .help("(WIP) By default, if a toolchain file is found, it is used as the MSRV. If this flag is provided, toolchain files will be ignored.")
                       //                            .takes_value(false),
                       //                    )
            ),
        )
}
