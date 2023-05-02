use actix_web::{post, web, App, HttpResponse, HttpServer, Responder};
use clap::Parser;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use verifier::read;
use verifier::{instance::Instance, solution::Solution};

type InstancesDb = HashMap<String, Instance>;

struct Db {
    instances: InstancesDb,
}

#[post("/check")]
async fn checker(data: web::Data<Db>, req_body: String) -> impl Responder {
    let solution = Solution::from_str(&req_body);

    match solution {
        Err(err) => HttpResponse::BadRequest().body(err),
        Ok(sol) => {
            let instance_name = &sol.instance_name;

            match data.instances.get(instance_name) {
                None => HttpResponse::BadRequest()
                    .body(format!("No such instance: `{}'", instance_name)),
                Some(instance) => match verifier::verify::verify(&instance, &sol) {
                    Ok(dist) => HttpResponse::Ok().body(dist.to_string()),
                    Err(err) => HttpResponse::Ok().body(err),
                },
            }
        }
    }
}

fn read_instances(instances_dir: &Path) -> InstancesDb {
    let mut db = InstancesDb::new();
    for fd in instances_dir.read_dir().unwrap() {
        let path = fd.unwrap().path();
        let instance = read::<Instance>(&path).unwrap();
        let instance_name = path.file_name().unwrap().to_str().unwrap().to_string();
        db.entry(instance_name).or_insert(instance);
    }

    println!("read {} instances", db.len());

    db
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// path to the location of instances to read
    #[arg(short, long)]
    instances_dir: PathBuf,

    /// port to bind to
    #[arg(short, long, default_value_t = 8080)]
    port: u16,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();

    println!("starting, listening on {}", args.port);
    let db = read_instances(&args.instances_dir);
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
