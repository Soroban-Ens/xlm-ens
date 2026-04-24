use clap::CommandFactory;
use clap_complete::{generate, Shell};
use std::io;

pub fn run_completions<C: CommandFactory>(shell: Shell, bin_name: &str) {
    let mut cmd = C::command();
    generate(shell, &mut cmd, bin_name, &mut io::stdout());
}
