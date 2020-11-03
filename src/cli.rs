use anyhow::*;
use clap::{clap_app, crate_version, App, ArgMatches};

fn app() -> App<'static, 'static> {
    clap_app!(cepler =>
        (version: crate_version!())
        (@setting VersionlessSubcommands)
        (@arg FILES: ... * "Input files")
    )
}

pub fn run() -> Result<()> {
    let matches = app().get_matches();
    if let Some(files) = matches.values_of("FILES") {
        println!("FILES: {:?}", files);
    } else {
        println!("NO FILES");
    }
    Ok(())
}
