use actix_web::{post, web, App, HttpResponse, HttpServer, Responder};
use clap::Parser;
use rug;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use verifier::read;
use verifier::verify::verify;
use verifier::instance::{fl, Instance};
use verifier::solution::Solution;
use walkdir;

type InstancesDb = HashMap<String, Instance>;

struct Db {
    instances: InstancesDb,
}

#[post("/check")]
async fn checker(data: web::Data<Db>, req_body: String) -> impl Responder {
    match Solution::from_str(&req_body) {
        Err(err) => HttpResponse::BadRequest().body(err),
        Ok(solution) => match data.instances.get(&solution.instance_name) {
            None => {
                let resp = format!("No such instance: `{}'", &solution.instance_name);

                HttpResponse::BadRequest().body(resp)
            }
            Some(instance) => {
                let resp = match verify(&instance, &solution) {
                    Ok(dist) => format!("{} {} {}", instance.name, solution.routes.len(), dist),
                    Err(err) => err,
                };

                HttpResponse::Ok().body(resp)
            }
        },
    }
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

#[derive(Debug)]
struct Bks {
    routes: usize,
    distance: rug::Float,
    // date
    // who
}

impl Bks {
    fn new() -> Self {
        Bks {
            routes: usize::MAX,
            distance: fl(0),
        }
    }
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

    let mut bks: HashMap<String, Bks> = HashMap::new();

    if let Some(bks_dir) = args.bks_dir {
        for b in walkdir::WalkDir::new(bks_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|f| f.file_type().is_file())
            .filter(|f| fs::metadata(f.path()).unwrap().len() > 0)
        {
            let sol = read::<Solution>(b.path()).unwrap();
            let inst = db.get(&sol.instance_name).unwrap();
            *bks.entry(sol.instance_name).or_insert(Bks::new()) = Bks { routes: sol.routes.len(), distance: verify(&inst, &sol).unwrap() };
        }
    }

    println!("read {} bks", bks.len());

    for (name, b) in bks.iter() {
        println!("{:10} : {:3} {}", name, b.routes, b.distance);
    }

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(Db {
                instances: db.clone(),
            }))
            .service(checker)
    })
    .bind(("127.0.0.1", args.port))?
    .run()
    .await
}
