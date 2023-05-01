use std::env;
use std::path::PathBuf;
use verifier;

struct Args {
    solution_path: PathBuf,
    instances_location: PathBuf,
}

impl Args {
    fn from_env() -> Option<Self> {
        let args: Vec<_> = env::args().collect();
        if args.len() == 1 {
            None
        } else {
            Some(Args {
                solution_path: PathBuf::from(&args[1]),
                instances_location: PathBuf::from(if args.len() < 3 { "." } else { &args[2] }),
            })
        }
    }
}

fn usage() {
    println!("verifier path_to_solution [path_to_instance_directory|path_to_instance]");
}

fn main() -> Result<(), String> {
    let args = Args::from_env();
    match args {
        None => {
            usage();
            Err("Not enough arguments".to_string())
        }
        Some(args) =>  {
            let (sol, res) = verifier::check_sintef_file(&args.solution_path, &args.instances_location)?;

            println!("{} {} {}", sol.instance_name, sol.routes.len(), res);
            Ok(())
        }
    }
}
