use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "bground")]
#[command(
    about = "CLI claim-grounding checker. Reads claim plus evidence; emits proceed-or-stop directive."
)]
pub struct BgroundCli {
    #[command(subcommand)]
    pub cmd: Cmd,
}

#[derive(Debug, Subcommand)]
pub enum Cmd {
    Verify(VerifyArgs),
    ClaimTypes,
    Update,
    Init,
    Tail,
    Explain,
}

#[derive(Debug, Clone, Args, Eq, PartialEq)]
pub struct VerifyArgs {
    pub claim: String,
    #[arg(long = "evidence", value_parser = parse_evidence_pair)]
    pub evidence: Vec<(String, String)>,
    #[arg(long)]
    pub manifest: Option<PathBuf>,
    #[arg(long)]
    pub json: bool,
    #[arg(long)]
    pub quiet: bool,
    #[arg(long)]
    pub reason: Option<String>,
}

fn parse_evidence_pair(value: &str) -> Result<(String, String), String> {
    let (key, item) = value
        .split_once('=')
        .ok_or_else(|| "expected <id>=<value>".to_owned())?;

    if key.trim().is_empty() {
        return Err("evidence id must not be empty".to_owned());
    }

    Ok((key.to_owned(), item.to_owned()))
}
