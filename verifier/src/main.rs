use std::env;
use verifier;
use std::path::Path;

fn main() -> Result<(), String> {
    let args : Vec<_> = env::args().collect();
    let sol_path = Path::new(&args[1]);
    let (sol, res) = verifier::check_sintef_file(sol_path, Path::new("./"))?;

    println!("{} {} {}", sol.instance_name, sol.routes.len(), res);
    Ok(())
}
