pub mod instance;
use std::str::FromStr;

use std::fs::read_to_string;

pub fn read_instance(path: &str) -> Result<instance::Instance, String> {
    let f = match read_to_string(path) {
        Ok(f) => f,
        Err(x) => {
            return Err(x.to_string());
        }
    };

    instance::Instance::from_str(&f)
}
