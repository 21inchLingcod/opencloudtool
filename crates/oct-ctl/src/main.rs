use actix_web::{get, middleware::Logger, post, web, App, HttpServer, Responder};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::process::Command;

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
        }
        Err(err) => {
            println!("{}", err);
            "Error"
        }
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
        .args([
            "run",
            "-d",
            "-p",
            format!("{port}:80", port = &obj.port).as_str(),
            &obj.image_uri.as_str(),
        ])
        .output();

    log::info!(
        "{}",
        String::from_utf8_lossy(&command.as_ref().expect("failed").stdout)
    );

    match command {
        Ok(res) => {
            println!("Result: {}", String::from_utf8_lossy(&res.stdout));
            "Success"
        }
        Err(err) => {
            println!("{}", err);
            "Error"
        }
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

// TODO: add tests
