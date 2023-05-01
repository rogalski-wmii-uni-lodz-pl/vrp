use std::env;
use std::path::PathBuf;
use verifier;

struct Args {
    solution_path: PathBuf,
    instances_location: PathBuf,
}

impl Args {
    fn from_env() -> Self {
        let args: Vec<_> = env::args().collect();
        Args {
            solution_path: PathBuf::from(&args[1]),
            instances_location: PathBuf::from(if args.len() < 2 { "." } else { &args[2] }),
        }
    }
}

fn main() -> Result<(), String> {
    let args = Args::from_env();
    let (sol, res) = verifier::check_sintef_file(&args.solution_path, &args.instances_location)?;

    println!("{} {} {}", sol.instance_name, sol.routes.len(), res);
    Ok(())
}
