pub mod verify;

pub use verify::instance;
pub use verify::solution;

use rug;
use std::fs::read_to_string;
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;

fn read<T: FromStr<Err = String>>(path: &Path) -> Result<T, String> {
    let f = read_to_string(path).map_err(|x| format!("{}: {x}", path.display()))?;

    T::from_str(&f)
}

pub fn check_sintef_file(
    path: &Path,
    instances_loc: &Path,
) -> Result<(solution::Solution, rug::Float), String> {
    let solution = read::<solution::Solution>(path)?;
    let instance_path = if instances_loc.is_dir() {
        instances_loc.join(&solution.instance_name)
    } else {
        PathBuf::from(instances_loc)
    };
    let instance = read::<instance::Instance>(&instance_path)?;
    let dist = verify::verify(&instance, &solution)?;

    Ok((solution, dist))
}
