use bground::{BgroundCli, ClaimType, Cmd, verify};
use bsuite_core::ExitCode;
use clap::Parser;

fn main() -> std::process::ExitCode {
    let cli = BgroundCli::parse();

    match run(cli) {
        Ok(exit_code) => exit_code,
        Err(error) => {
            eprintln!("{error}");
            process_exit_code(ExitCode::Usage)
        }
    }
}

fn run(cli: BgroundCli) -> Result<std::process::ExitCode, bground::BgroundError> {
    match cli.cmd {
        Cmd::ClaimTypes => {
            for claim_type in ClaimType::ALL {
                println!("{claim_type}");
            }
        }
        Cmd::Verify(args) => verify::validate_args(&args)?,
        Cmd::Update | Cmd::Init | Cmd::Tail | Cmd::Explain => {}
    }

    Ok(process_exit_code(ExitCode::Success))
}

fn process_exit_code(exit_code: ExitCode) -> std::process::ExitCode {
    std::process::ExitCode::from(exit_code.as_i32() as u8)
}
