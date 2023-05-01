use actix_web::{post, web, App, HttpResponse, HttpServer, Responder};
use std::collections::HashMap;
use std::path::Path;
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

    let sol = solution.unwrap();
    let instance = data.instances.get(&sol.instance_name).unwrap();

    HttpResponse::Ok().body(format!(
        "{}",
        verifier::verify::verify(&instance, &sol).unwrap()
    ))
}

fn read_instances() -> InstancesDb {
    let mut db = InstancesDb::new();
    for fd in Path::new("./i/").read_dir().unwrap() {
        let path = fd.unwrap().path();
        let instance = read::<Instance>(&path).unwrap();
        let instance_name = path.file_name().unwrap().to_str().unwrap().to_string();
        db.entry(instance_name).or_insert(instance);
    }

    println!("read {} instances", db.len());

    db
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port = 8080;
    println!("starting, listening on {port}");
    let db = read_instances();
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(Db {
                instances: db.clone(),
            }))
            .service(checker)
    })
    .bind(("127.0.0.1", port))?
    .run()
    .await
}
