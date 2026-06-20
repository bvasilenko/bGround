mod invocation;
mod runtime;

use bground::{BgroundCli, BgroundError, ClaimType, Cmd};
use bsuite_core::{
    BsuiteCoreError, EmitFormat, ExitCode, ProcessExitEmitter, prompt_resolver::DirectiveString,
};
use clap::Parser;
use invocation::InvocationTranscript;
use runtime::BinaryRuntime;
use std::path::PathBuf;

fn main() {
    let cli = BgroundCli::parse();
    let format = emit_format_for(&cli.cmd);
    let mut emitter = ProcessExitEmitter::new(format);

    let exit_code = match init_and_run(cli) {
        Ok(CommandOutcome::Directive {
            directive,
            exit_code,
        }) => emitter.emit_directive(Ok((directive, exit_code))),
        Ok(CommandOutcome::Silent(exit_code)) => exit_code,
        Err(RunError::Malformed(e)) => {
            eprintln!("{e}");
            ExitCode::Usage
        }
        Err(RunError::Internal(e)) => emitter.emit_directive(Err(e)),
    };

    std::process::exit(exit_code.as_i32());
}

fn init_and_run(cli: BgroundCli) -> Result<CommandOutcome, RunError> {
    let runtime = BinaryRuntime::init(install_dir()).map_err(RunError::Internal)?;
    let invocation = InvocationTranscript::start(
        runtime.host_context,
        runtime.invocation_context.clone(),
        runtime.corpus_version,
    );
    run(cli, runtime, invocation)
}

fn run(
    cli: BgroundCli,
    runtime: BinaryRuntime,
    invocation: InvocationTranscript,
) -> Result<CommandOutcome, RunError> {
    match cli.cmd {
        Cmd::Verify(args) => {
            let result = bground::verify::run(&args, &runtime.corpus, runtime.host_context)
                .map_err(classify_bground_error);
            let exit_code = result
                .as_ref()
                .map_or_else(|e| e.exit_code(), |(_, code)| *code);
            invocation.flush(&runtime.appender, exit_code, result.is_ok());
            result.map(|(directive, exit_code)| CommandOutcome::Directive {
                directive,
                exit_code,
            })
        }

        Cmd::ClaimTypes => {
            let listing = ClaimType::ALL
                .iter()
                .map(|ct| ct.to_string())
                .collect::<Vec<_>>()
                .join("\n");
            invocation.flush(&runtime.appender, ExitCode::Success, false);
            Ok(CommandOutcome::Directive {
                directive: DirectiveString::new(listing),
                exit_code: ExitCode::Success,
            })
        }

        Cmd::Update => {
            let result = bground::update::run(&runtime.install_dir)
                .map_err(|e| RunError::Internal(e.into_core()));
            let exit_code = result
                .as_ref()
                .map_or_else(|e| e.exit_code(), |()| ExitCode::Success);
            invocation.flush(&runtime.appender, exit_code, false);
            result.map(|()| CommandOutcome::Silent(ExitCode::Success))
        }

        Cmd::Init | Cmd::Tail | Cmd::Explain => {
            invocation.flush(&runtime.appender, ExitCode::Success, false);
            Ok(CommandOutcome::Silent(ExitCode::Success))
        }
    }
}

fn emit_format_for(cmd: &Cmd) -> EmitFormat {
    match cmd {
        Cmd::Verify(args) if args.json => EmitFormat::Json,
        _ => EmitFormat::Plain,
    }
}

fn install_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from("."))
}

fn classify_bground_error(e: BgroundError) -> RunError {
    if e.is_malformed_input() {
        RunError::Malformed(e)
    } else {
        RunError::Internal(e.into_core())
    }
}

#[derive(Debug)]
enum CommandOutcome {
    Directive {
        directive: DirectiveString,
        exit_code: ExitCode,
    },
    Silent(ExitCode),
}

#[derive(Debug)]
enum RunError {
    Malformed(BgroundError),
    Internal(BsuiteCoreError),
}

impl RunError {
    fn exit_code(&self) -> ExitCode {
        match self {
            Self::Malformed(_) => ExitCode::Usage,
            Self::Internal(_) => ExitCode::InternalError,
        }
    }
}
