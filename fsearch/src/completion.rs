use clap::Command;

use clap_complete::shells::{Bash, Elvish, Fish, Zsh};
use clap_complete::{generate, Shell};
use std::io;

fn get_cmd() -> Command {
    Command::new("fsearch")
        .about("A spotlight like search util for linux")
        .version("0.0.1")
        .author("luxluth <delphin.blehoussi93@gmail.com>")
        .subcommand(Command::new("daemon").about("Start the fsearch daemon"))
        .subcommand(Command::new("status").about("Get daemon status"))
        .subcommand(Command::new("stop").about("Stop the fsearch daemon"))
        .subcommand(
            Command::new("config")
                .about("Apply specific configuration to fsearch")
                .arg(
                    clap::Arg::new("config")
                        .short('c')
                        .long("config")
                        .value_name("FILE_PATH")
                        .help("The path of the config file .toml"),
                )
                .arg(
                    clap::Arg::new("css")
                        .short('s')
                        .long("css")
                        .value_name("FILE_PATH")
                        .help("The path of the css file"),
                ),
        )
        .subcommand(
            Command::new("completion")
                .about("Generate command completion")
                .arg(
                    clap::Arg::new("shell")
                        .short('s')
                        .long("shell")
                        .value_name("SHELL")
                        .help("The shell to generate the completion for"),
                ),
        )
}

pub fn generate_completion(shell: Shell) {
    let mut cmd = get_cmd();
    match shell {
        Shell::Bash => {
            generate::<Bash, _>(Bash, &mut cmd, "fsearch", &mut io::stdout());
        }
        Shell::Zsh => {
            generate::<Zsh, _>(Zsh, &mut cmd, "fsearch", &mut io::stdout());
        }
        Shell::Fish => {
            generate::<Fish, _>(Fish, &mut cmd, "fsearch", &mut io::stdout());
        }
        Shell::Elvish => {
            generate::<Elvish, _>(Elvish, &mut cmd, "fsearch", &mut io::stdout());
        }

        _ => {
            println!("Unsupported shell");
        }
    }
}
