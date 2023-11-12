use clap::Parser;

mod variant;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Name of the profile variant to use.
    #[arg(short, long)]
    name: String,

    /// Bakes the current profile into the current repository unless
    /// name is specified.
    #[arg(short, long)]
    sacred: bool,
}

fn main() {
    let args = Args::parse();

    if args.sacred {
        match variant::bake_variant(&args.name) {
            Ok(()) => {
                print!("Baked profile {}", args.name);
            }
            Err(err) => {
                eprint!("Failed to bake profile {}: {}", args.name, err)
            }
        }
    }

    match variant::set_profile(&args.name) {
        Ok(()) => {
            print!("Set profile {}", args.name);
        }
        Err(err) => {
            eprint!("Failed to set profile {}: {}", args.name, err)
        }
    }
}
