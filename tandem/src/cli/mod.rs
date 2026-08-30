//! Clap-based command-line interface.
mod commands;
mod landing;
mod model;
mod output;
mod parse;

use crate::CliError;

pub(crate) enum StartupRequest {
    Exit,
    Tui,
    Web(crate::web::Options),
}

pub(crate) fn run(args: Vec<String>) -> Result<StartupRequest, CliError> {
    if args.is_empty() {
        landing::print();
        return Ok(StartupRequest::Exit);
    }
    let json = parse::json_requested(&args);
    let cli = match parse::parse(&args) {
        Ok(cli) => cli,
        Err(error) if parse::is_help(&error) => {
            println!("{error}");
            return Ok(StartupRequest::Exit);
        }
        Err(error) => return Err(output::usage_error(error.to_string(), json)),
    };
    let Some(command) = cli.command else {
        landing::print();
        return Ok(StartupRequest::Exit);
    };
    commands::dispatch(command, cli.json).map_err(|error| error.with_json(json))
}
