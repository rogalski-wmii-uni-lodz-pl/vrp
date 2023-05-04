use actix_web::http::header::ContentType;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use clap::Parser;
use rug;
use serde::{ser::SerializeStruct, Deserialize, Serialize};
use std::cmp::Ordering;
use std::ops::Sub;
use std::path::PathBuf;
use std::str::FromStr;
use verifier::instance::flf64;
use verifier::solution::Solution;
use verifier::verify::verify;

mod data;
use data::{Bks, Db};

#[derive(Debug)]
struct Verification {
    instance_name: String,
    routes: usize,
    distance: rug::Float,
}

impl Serialize for Verification {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Verification", 3)?;
        state.serialize_field("instance_name", &self.instance_name)?;
        state.serialize_field("routes", &self.routes)?;
        state.serialize_field("distance", &self.distance.to_string())?;
        state.end()
    }
}

impl ToString for Verification {
    fn to_string(&self) -> String {
        format!("{}, {}, {}", self.instance_name, self.routes, self.distance)
    }
}

#[derive(Debug)]
struct VerificationWithComparison {
    verification: Verification,
    comparison: Ordering,
    bks: Option<Bks>,
}

impl ToString for VerificationWithComparison {
    fn to_string(&self) -> String {
        format!(
            "{} {} {}",
            self.verification.to_string(),
            format_comparison(self.comparison),
            match &self.bks {
                None => "None".to_string(),
                Some(b) => format!("{:?}", b),
            }
        )
    }
}

fn format_comparison(ord: Ordering) -> String {
    match ord {
        Ordering::Less => "better than",
        Ordering::Equal => "equal to",
        Ordering::Greater => "ok",
    }
    .to_string()
}

impl Serialize for VerificationWithComparison {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("VerificationWithComparison", 3)?;
        state.serialize_field("verification", &self.verification)?;
        state.serialize_field("comparision", &format_comparison(self.comparison))?;
        state.serialize_field("bks", &self.bks)?;
        state.end()
    }
}

fn check(db: &web::Data<Db>, sol: &Solution) -> Result<VerificationWithComparison, String> {
    let inst = db.instance(&sol.instance_name)?;
    let best = db.bks(&sol.instance_name).map(|bs| bs.last().cloned())?;

    let verification = verify(inst, &sol).map(|dist| Verification {
        instance_name: inst.name.clone(),
        routes: sol.routes.len(),
        distance: dist,
    })?;

    Ok(compare(verification, best))
}

fn compare(verification: Verification, best: Option<Bks>) -> VerificationWithComparison {
    let ord = match &best {
        None => Ordering::Less,
        Some(best) => {
            match verification.routes.cmp(&best.routes) {
                Ordering::Equal => {
                    let diff = best.distance.clone().sub(&verification.distance);
                    if diff < flf64(-0.001) {
                        Ordering::Less
                    } else if diff.abs() < flf64(0.001) {
                        Ordering::Equal
                    } else {
                        Ordering::Greater
                    }
                }
                // Less => Less,
                // Greater => Greater,
                less_or_greater => less_or_greater,
            }
        }
    };

    VerificationWithComparison {
        verification,
        comparison: ord,
        bks: best,
    }
}

fn resp(resp: Result<String, String>) -> HttpResponse {
    match resp {
        Err(err) => HttpResponse::BadRequest().body(err),
        Ok(resp) => HttpResponse::Ok().body(resp),
    }
}

#[derive(Serialize, Deserialize)]
struct Error {
    err: String,
}

fn resp_json<T: Serialize>(resp: Result<T, String>) -> HttpResponse {
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
        Ok(sol) => resp(check(&db, &sol).map(|x| x.to_string())),
    }
}

#[get("/instance/{instance}")]
async fn get_instance(db: web::Data<Db>, path: web::Path<String>) -> impl Responder {
    let name = path.into_inner();
    resp(db.instance(&name).map(|inst| inst.to_string()))
}

#[get("/history/{instance}")]
async fn get_bks_history(db: web::Data<Db>, path: web::Path<String>) -> impl Responder {
    let name = path.into_inner();
    resp(db.bks(&name).map(|bks| {
        bks.iter()
            .map(|x| format!("{:?}", x))
            .collect::<Vec<String>>()
            .join("\n")
    }))
}

#[post("/json/check")]
async fn json_checker(db: web::Data<Db>, req_body: web::Json<Solution>) -> impl Responder {
    resp_json(check(&db, &req_body))
}

#[get("/json/history/{instance}")]
async fn json_bks_history(db: web::Data<Db>, path: web::Path<String>) -> impl Responder {
    let name = path.into_inner();
    resp_json(db.bks(&name))
}

#[get("/json/instance/{instance}")]
async fn get_json_instance(db: web::Data<Db>, path: web::Path<String>) -> impl Responder {
    resp_json(db.instance(&path.into_inner()))
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
    let db = Db::new(&args.instances_dir, &args.bks_dir)?;
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db.clone()))
            .service(checker)
            .service(json_checker)
            .service(get_instance)
            .service(get_json_instance)
            .service(get_bks_history)
            .service(json_bks_history)
    })
    .bind(("127.0.0.1", args.port))?
    .run()
    .await
}
