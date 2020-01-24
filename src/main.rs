#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate fomat_macros;

use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use futures::executor;
use pop_os_api::builds::Build;

fn main() {
    let matches = App::new("pop-os-api")
        .about("Tool for interacting with the Pop!_OS API")
        .global_setting(AppSettings::ColoredHelp)
        .global_setting(AppSettings::UnifiedHelpMessage)
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(
            SubCommand::with_name("builds")
                .about("fetch information on a build")
                .arg(Arg::with_name("version").takes_value(true).required(true))
                .arg(Arg::with_name("channel").takes_value(true).required(true)),
        )
        .get_matches();

    if let Err(why) = executor::block_on(main_(matches)) {
        eprintln!("pop-os-api: {:#?}", why);
        std::process::exit(1);
    }
}

async fn main_(matches: ArgMatches<'_>) -> Result<(), anyhow::Error> {
    match matches.subcommand() {
        ("builds", Some(matches)) => {
            let version = matches.value_of("version").expect("version required");
            let channel = matches.value_of("channel").expect("channel required");
            let build = Build::get(version, channel).await?;

            pint!(
                "build:   " (build.build) "\n"
                "sha_sum: " (build.sha_sum) "\n"
                "size:    " (build.size) "\n"
                "url:     " (build.url) "\n"
            );

            Ok(())
        }
        (sub, _) => Err(anyhow!("unknown subcommand: {}", sub)),
    }
}
