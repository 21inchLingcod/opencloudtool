use actix_web::{get, post, web, App, middleware::Logger, HttpServer,  Responder};
use std::process::Command;
use std::io::{self, Write};
use serde::{Deserialize, Serialize};
use futures::StreamExt;

#[get("/")]
async fn index() -> impl Responder {
    let command = Command::new("docker")
        .arg("run")
        .arg("-d")
        .arg("-p")
        .arg("80:80")
        .arg("nginx")
        .output();

    match command {
        Ok(res) => {
            println!("Result: {}", String::from_utf8_lossy(&res.stdout));
            "Success"
        },
        Err(err) => {
            println!("{}", err);
            "Error"
        },
    }
}

const MAX_SIZE: usize = 262_144; // max payload size is 256k
//
#[derive(Serialize, Deserialize)]
struct RunContainerPayload {
    image_uri: String,
    port: String,
}

#[post("/run-container")]
async fn run(mut payload: web::Payload) -> impl Responder {
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
            let chunk = chunk.unwrap();
            // limit max size of in-memory payload
            if (body.len() + chunk.len()) > MAX_SIZE {
                return "Payload too large";
            }
            body.extend_from_slice(&chunk);
    }

    let obj = serde_json::from_slice::<RunContainerPayload>(&body).unwrap();

    let command = Command::new("podman")
        .args(["run",
            "-d",
            "-p", 
            format!("{port}:{port}", port=&obj.port).as_str(),
            &obj.image_uri])
        .output();
        
    io::stdout().write_all(&command.as_ref().expect("failed").stdout).unwrap();

    match command {
        Ok(res) => {
            println!("Result: {}", String::from_utf8_lossy(&res.stdout));
            "Success"
        },
        Err(err) => {
            println!("{}", err);
            "Error"
        },
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    log::info!("Starting server at http://0.0.0.0:31888");

    HttpServer::new(|| {
        let logger = Logger::default();
        App::new().wrap(logger).service(index).service(run)
    })
        .bind(("0.0.0.0", 31888))?
        .run()
        .await
}
