use clap::ValueEnum;
use serde_json::Value;

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum OutputFormat {
    Human,
    Json,
}

pub fn emit(format: OutputFormat, human: &str, json: Value) {
    match format {
        OutputFormat::Human => println!("{human}"),
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(&json)
                    .expect("json output should always serialize")
            );
        }
    }
}

pub fn emit_error(format: OutputFormat, human: &str, json: Value) {
    match format {
        OutputFormat::Human => eprintln!("{human}"),
        OutputFormat::Json => {
            eprintln!(
                "{}",
                serde_json::to_string_pretty(&json)
                    .expect("json output should always serialize")
            );
        }
    }
}
