use std::process::{Command, ExitCode};

use clap::Parser;

mod api;
mod errors;
mod plugins;
use api::{get_git_remote_url, set_variant, variants, whoami};
use errors::VariantError;
use plugins::{Persist, persist::VariantConfig, prompt::input_prompt};

#[derive(Parser, Debug)]
#[command(author, version, about)]
#[non_exhaustive]
enum Commands {
    #[command(about = "Sets the git profile variant.")]
    Var {
        /// The name of the profile to use. e.g. `foo` or `bar` depending on the
        /// folder the config is in.
        #[arg(short, long)]
        name: String,
        /// Indicates that only the local config will be changed and the global config
        /// remains untouched. The project will be tied to this profile.
        #[arg(short, long, default_value_t = false)]
        sacred: bool,
    },

    #[command(about = "Lists all the git profile variants.")]
    List,

    #[command(about = "Provides the configured git profile information.")]
    Whoami {
        /// Provides all the data found in the git config without any truncation.
        #[arg(short, long, default_value_t = false)]
        verbose: bool,
    },

    #[command(about = "Wrapper around git commands. Validates profile is set before commits.")]
    Git {
        /// Git command and arguments
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },

    #[command(about = "Prints the version of the application.")]
    Version,
}

fn check_profile_set() -> Result<bool, VariantError> {
    let output = Command::new("git")
        .args(["config", "--get", "user.email"])
        .output()?;

    Ok(output.status.success() && !output.stdout.is_empty())
}

fn get_current_profile_username() -> Result<Option<String>, VariantError> {
    let name_output = Command::new("git")
        .args(["config", "--get", "user.name"])
        .output()?;

    let email_output = Command::new("git")
        .args(["config", "--get", "user.email"])
        .output()?;

    if !name_output.status.success()
        || name_output.stdout.is_empty()
        || !email_output.status.success()
        || email_output.stdout.is_empty()
    {
        return Ok(None);
    }

    let name = String::from_utf8_lossy(&name_output.stdout)
        .trim()
        .to_string();
    let email = String::from_utf8_lossy(&email_output.stdout)
        .trim()
        .to_string();

    let project_cache = VariantConfig::init()?;
    let all_profiles = project_cache.read_all()?;

    for profile in all_profiles {
        if profile.name == name && profile.email == email {
            return Ok(Some(profile.username));
        }
    }

    Ok(None)
}

fn check_sacred_association() -> Result<(), VariantError> {
    if let Some(remote_url) = get_git_remote_url()? {
        let project_cache = VariantConfig::init()?;
        if let Some(cached_profile) = project_cache.get_project_profile(&remote_url)?
            && let Some(current_profile) = get_current_profile_username()?
            && current_profile != cached_profile
        {
            return Err(VariantError::System(format!(
                "This repository is linked to profile '{}', but current profile is '{}'. Please run 'variant var -n {} --sacred' first.",
                cached_profile, current_profile, cached_profile
            )));
        }
    }
    Ok(())
}

fn main() -> ExitCode {
    match Commands::parse() {
        Commands::Whoami { verbose } => match whoami(verbose) {
            Ok(data) => {
                println!("{}", String::from_utf8_lossy(&data));
                ExitCode::SUCCESS
            }
            Err(data) => {
                eprintln!("{}", data);
                ExitCode::FAILURE
            }
        },

        Commands::List => match variants() {
            Ok(variants) => {
                for variant in variants {
                    println!("{}", variant.name);
                }
                ExitCode::SUCCESS
            }
            Err(data) => {
                eprintln!("{}", data);
                ExitCode::FAILURE
            }
        },

        Commands::Var { name, sacred } => {
            let persistance =
                VariantConfig::init().expect("must be able to initialize persistance");

            match set_variant(name, persistance, &input_prompt, sacred) {
                Ok(_) => {
                    println!("Successfully set variant.");
                    ExitCode::SUCCESS
                }
                Err(data) => {
                    eprintln!("{}", data);
                    ExitCode::FAILURE
                }
            }
        }

        Commands::Git { args } => {
            if args.is_empty() {
                let status = Command::new("git").status();
                return match status {
                    Ok(s) => ExitCode::from(s.code().unwrap_or(1) as u8),
                    Err(e) => {
                        eprintln!("Failed to execute git: {}", e);
                        ExitCode::FAILURE
                    }
                };
            }

            let git_command = &args[0];
            if matches!(git_command.as_str(), "commit" | "push" | "add") {
                match check_profile_set() {
                    Ok(true) => {}
                    Ok(false) => {
                        eprintln!(
                            "Error: No git profile set. Please run 'variant var -n <profile>' first."
                        );
                        return ExitCode::FAILURE;
                    }
                    Err(e) => {
                        eprintln!("Error checking git profile: {}", e);
                        return ExitCode::FAILURE;
                    }
                }

                if let Err(e) = check_sacred_association() {
                    eprintln!("{}", e);
                    return ExitCode::FAILURE;
                }
            }

            let status = Command::new("git").args(&args).status();
            match status {
                Ok(s) => ExitCode::from(s.code().unwrap_or(1) as u8),
                Err(e) => {
                    eprintln!("Failed to execute git: {}", e);
                    ExitCode::FAILURE
                }
            }
        }

        Commands::Version => {
            println!("{}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
    }
}
