use clap::Parser;

mod variant;

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
        /// Provides the log of the changes effected without any truncation.
        #[arg(short, long, default_value_t = false)]
        verbose: bool,
    },

    #[command(about = "Provides the configured git profile information.")]
    Whoami {
        /// Provides all the data found in the git config without any truncation.
        #[arg(short, long, default_value_t = false)]
        verbose: bool,
    },
}

fn main() {
    match Commands::parse() {
        Commands::Whoami { verbose } => match variant::whoami(verbose) {
            Ok(data) => {
                println!("{}", String::from_utf8_lossy(&data));
            }
            Err(data) => {
                eprintln!("{}", String::from_utf8_lossy(&data));
            }
        },
        Commands::Var {
            name,
            sacred,
            verbose,
        } => {
            // FIXME
            println!(
                "profile with name: {}, sacred: {}, verbose: {}",
                name, sacred, verbose
            );
        }
    }
}
