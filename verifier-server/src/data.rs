use chrono::NaiveDate;
use serde::Serialize;
use serde_with::{serde_as, DisplayFromStr};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use walkdir;

use verifier::instance::{flf64, Instance};
use verifier::read;
use verifier::solution::Solution;
use verifier::verify::verify;

pub type Instances = HashMap<String, Instance>;

pub fn read_instances(instances_dir: &Path) -> Result<Instances, std::io::Error> {
    let mut db = Instances::new();

    for fd in instances_dir.read_dir()? {
        let path = fd.unwrap().path();
        match read::<Instance>(&path) {
            Ok(instance) => {
                let instance_name = path.file_name().unwrap().to_str().unwrap().to_string();
                db.entry(instance_name).or_insert(instance);
            }
            Err(err) => println!("{}: {err}", path.display()),
        }
    }

    println!("read {} instances", db.len());

    Ok(db)
}

#[serde_as]
#[derive(Debug, Clone, Serialize)]
pub struct Bks {
    pub routes: usize,
    #[serde_as(as = "DisplayFromStr")]
    pub distance: rug::Float,
    #[serde_as(as = "DisplayFromStr")]
    pub date: NaiveDate,
    pub solution: Option<Solution>,
    // who
}

type BksDb = HashMap<String, Vec<Bks>>;

pub fn read_bks(instances: &Instances, bks_dir: &Option<PathBuf>) -> Result<BksDb, std::io::Error> {
    let mut bks: HashMap<String, Vec<Bks>> = HashMap::new();

    if let Some(bks_dir) = bks_dir {
        for b in walkdir::WalkDir::new(bks_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|f| f.file_type().is_file())
        {
            let date = get_date_from_parent_dir(&b);
            let (name, best) = create_bks(&b, instances, date);
            (*bks.entry(name).or_insert(vec![])).push(best);
        }
    }

    println!("read {} bks", bks.len());

    // for (name, b) in bks.iter() {
    //     let bl = b.last().unwrap();
    //     println!("{} {:10} : {:3} {}", bl.date, name, bl.routes, bl.distance);
    // }

    Ok(bks)
}

fn create_bks(b: &walkdir::DirEntry, instances: &Instances, date: NaiveDate) -> (String, Bks) {
    let empty_file = fs::metadata(b.path()).unwrap().len() == 0;

    let (name, routes, distance, solution) = if empty_file {
        extract_from_file_name(&b)
    } else {
        calculate(&b, instances)
    };

    (
        name,
        Bks {
            routes,
            distance,
            date,
            solution,
        },
    )
}

fn calculate(
    b: &walkdir::DirEntry,
    instances: &Instances,
) -> (String, usize, rug::Float, Option<Solution>) {
    let sol = read::<Solution>(b.path()).unwrap();
    let inst = instances.get(&sol.instance_name).unwrap();

    (
        sol.instance_name.clone(),
        sol.routes.len(),
        verify(&inst, &sol).unwrap(),
        Some(sol),
    )
}

fn extract_from_file_name(b: &walkdir::DirEntry) -> (String, usize, rug::Float, Option<Solution>) {
    let (inst, rest) = b
        .path()
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .split_once('.')
        .unwrap();

    let (routes_quality, _) = rest.rsplit_once('.').unwrap();
    let (routes, quality) = routes_quality.split_once('_').unwrap();

    (
        inst.to_string(),
        routes.parse::<usize>().unwrap(),
        flf64(quality.parse::<f64>().unwrap()),
        None,
    )
}

fn get_date_from_parent_dir(b: &walkdir::DirEntry) -> NaiveDate {
    NaiveDate::from_str(
        b.path()
            .parent()
            .unwrap()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap(),
    )
    .unwrap()
}

#[derive(Clone)]
pub struct Db {
    instances: Instances,
    bks: BksDb,
}

impl Db {
    pub fn instance(&self, name: &String) -> Result<&Instance, String> {
        match self.instances.get(name) {
            None => Err(format!("No such instance: `{}'", name)),
            Some(instance) => Ok(&instance),
        }
    }

    pub fn bks(&self, name: &String) -> Result<&Vec<Bks>, String> {
        match self.bks.get(name) {
            None => Err(format!("No such instance: `{}'", name)),
            Some(b) => Ok(&b),
        }
    }

    pub fn new(instances_dir: &PathBuf, bks_dir: &Option<PathBuf>) -> std::io::Result<Self> {
        let instances = read_instances(instances_dir)?;
        let bks = read_bks(&instances, bks_dir)?;
        Ok(Self { instances, bks })
    }
}
