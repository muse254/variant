use clap::Parser;

mod api;
mod plugins;
use plugins::{persist::VariantConfig, prompt::input_prompt};

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
        /// remains untouched.
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
}

fn main() {
    match Commands::parse() {
        Commands::Whoami { verbose } => match api::whoami(verbose) {
            Ok(data) => {
                println!("{}", String::from_utf8_lossy(&data));
            }
            Err(data) => {
                eprintln!("{}", String::from_utf8_lossy(&data));
            }
        },

        Commands::List => match api::variants() {
            Ok(variants) => {
                for variant in variants {
                    println!("{}", variant.name);
                }
            }
            Err(data) => {
                eprintln!("{}", String::from_utf8_lossy(&data));
            }
        },

        Commands::Var { name, sacred } => {
            let persistance =
                VariantConfig::init().expect("must be able to initialize persistance");

            match api::set_variant(name, persistance, &input_prompt, sacred) {
                Ok(_) => {
                    println!("Successfully set variant.");
                }
                Err(data) => {
                    eprintln!("{}", String::from_utf8_lossy(&data));
                }
            }
        }
    }
}
