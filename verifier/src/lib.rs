pub mod verify;

pub use verify::instance;
pub use verify::solution;

use std::fs::read_to_string;
use std::path::Path;
use std::str::FromStr;
use rug;

fn read<T : FromStr<Err=String>>(path: &Path) -> Result<T, String> {
    let f = read_to_string(path).map_err(|x| x.to_string())?;

    T::from_str(&f)
}


pub fn check_sintef_file(path: &Path, instances_loc: &Path) -> Result<rug::Float, String> {
    let solution = read::<solution::Solution>(path)?;
    let instance_path = instances_loc.join(&solution.instance_name);
    let instance = read::<instance::Instance>(&instance_path)?;
    let dist = verify::verify(&instance, &solution)?;

    Ok(dist)
}
