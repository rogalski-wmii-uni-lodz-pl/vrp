use actix_web::http::header::ContentType;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use chrono::NaiveDate;
use clap::Parser;
use rug;
use serde;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use verifier::instance::{flf64, Instance};
use verifier::read;
use verifier::solution::Solution;
use verifier::verify::verify;
use walkdir;

type InstancesDb = HashMap<String, Instance>;

struct Db {
    instances: InstancesDb,
    bks: BksDb,
}

impl Db {
    fn instance(&self, name: &String) -> Result<&Instance, String> {
        match self.instances.get(name) {
            None => Err(format!("No such instance: `{}'", name)),
            Some(instance) => Ok(&instance),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Verification {
    instance_name: String,
    routes: usize,
    distance: String,
}

fn check(db: &web::Data<Db>, sol: &Solution) -> Result<(String, usize, rug::Float), String> {
    let inst = db.instance(&sol.instance_name)?;
    verify(inst, &sol).map(|dist| (inst.name.clone(), sol.routes.len(), dist))
}

fn resp(resp: Result<String, String>) -> HttpResponse {
    match resp {
        Err(err) => HttpResponse::BadRequest().body(err),
        Ok(resp) => HttpResponse::Ok().body(resp),
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Error {
    err: String,
}

fn resp_json<T: serde::Serialize>(resp: Result<T, String>) -> HttpResponse {
    match resp {
        Err(err) => HttpResponse::BadRequest()
            .content_type(ContentType::json())
            .body(serde_json::to_string(&Error { err }).unwrap()),
        Ok(resp) => HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(serde_json::to_string(&resp).unwrap()),
    }
}

#[post("/check")]
async fn checker(db: web::Data<Db>, req_body: String) -> impl Responder {
    match Solution::from_str(&req_body) {
        Err(err) => HttpResponse::BadRequest().body(err),
        Ok(sol) => resp(check(&db, &sol).map(|(i, r, d)| { match db.bks.get(&i) {
            Some(bks) => format!("{i} {r} {d} {:?}", bks.last().unwrap()),
            None => format!("{i} {r} {d}"),
        }})),
    }
}

#[get("/instance/{instance}")]
async fn get_instance(db: web::Data<Db>, path: web::Path<String>) -> impl Responder {
    let name = path.into_inner();
    resp(db.instance(&name).map(|inst| inst.to_string()))
}

#[post("/json/check")]
async fn json_checker(db: web::Data<Db>, req_body: web::Json<Solution>) -> impl Responder {
    resp_json(
        check(&db, &req_body).map(|(instance_name, routes, d)| Verification {
            instance_name,
            routes,
            distance: d.to_string(),
        }),
    )
}

#[get("/json/instance/{instance}")]
async fn get_json_instance(db: web::Data<Db>, path: web::Path<String>) -> impl Responder {
    resp_json(db.instance(&path.into_inner()))
}

fn read_instances(instances_dir: &Path) -> Result<InstancesDb, std::io::Error> {
    let mut db = InstancesDb::new();

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

#[derive(Debug, Clone)]
struct Bks {
    routes: usize,
    distance: rug::Float,
    date: NaiveDate,
    // who
}

type BksDb = HashMap<String, Vec<Bks>>;

fn read_bks(db: &InstancesDb, bks_dir: &Option<PathBuf>) -> Result<BksDb, std::io::Error> {
    let mut bks: HashMap<String, Vec<Bks>> = HashMap::new();

    if let Some(bks_dir) = bks_dir {
        for b in walkdir::WalkDir::new(bks_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|f| f.file_type().is_file())
        {
            let date = b
                .clone()
                .into_path()
                .parent()
                .unwrap()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string();

            let (name, routes, distance) = if fs::metadata(b.path()).unwrap().len() > 0 {
                let sol = read::<Solution>(b.path()).unwrap();
                let inst = db.get(&sol.instance_name).unwrap();

                (
                    sol.instance_name.clone(),
                    sol.routes.len(),
                    verify(&inst, &sol).unwrap(),
                )
            } else {
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
                )
            };

            (*bks.entry(name).or_insert(vec![])).push(Bks {
                routes,
                distance,
                date: NaiveDate::from_str(&date).unwrap(),
            });
        }
    }

    println!("read {} bks", bks.len());

    for (name, b) in bks.iter() {
        let bl = b.last().unwrap();
        println!("{} {:10} : {:3} {}", bl.date, name, bl.routes, bl.distance);
    }

    Ok(bks)
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// path to the directory containing instances
    #[arg(short, long)]
    instances_dir: PathBuf,

    /// path to the directory containing best known solutions
    #[arg(short, long)]
    bks_dir: Option<PathBuf>,

    /// port to bind to
    #[arg(short, long, default_value_t = 8080)]
    port: u16,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();

    println!("starting, listening on {}", args.port);
    let db = read_instances(&args.instances_dir)?;

    let bks = read_bks(&db, &args.bks_dir)?;

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(Db {
                instances: db.clone(),
                bks: bks.clone(),
            }))
            .service(checker)
            .service(json_checker)
            .service(get_instance)
            .service(get_json_instance)
    })
    .bind(("127.0.0.1", args.port))?
    .run()
    .await
}
