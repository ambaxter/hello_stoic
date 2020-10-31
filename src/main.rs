use actix_files as fs;
use actix_web::http::{header, Method, StatusCode};
use actix_web::{
    error, get, guard, middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer, Result,
};
use lazy_static::lazy_static;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::{env, io};
use std::path::PathBuf;
use actix_web::dev::Service;
use gethostname::gethostname;
use structopt::StructOpt;
mod texts;

/// favicon handler
#[get("/favicon.ico")]
async fn favicon() -> Result<fs::NamedFile> {
    Ok(fs::NamedFile::open("static/favicon.ico")?)
}

/// health handler
#[get("/health")]
async fn health() -> Result<HttpResponse> {
    lazy_static! {
        static ref HEALTH_STAT: AtomicUsize = AtomicUsize::new(0);
    }
    let current_health = HEALTH_STAT.fetch_add(1, Ordering::Relaxed);
    if current_health < 5 {
        Ok(HttpResponse::Ok()
            .content_type("text/plain; charset=utf-8")
            .body(format!("Ok - {}", current_health)))
    } else {
        Ok(HttpResponse::InternalServerError()
            .content_type("text/plain; charset=utf-8")
            .body("Definitely Not Ok"))
    }
}

/// health handler
#[get("/liveness")]
async fn liveness() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok()
        .content_type("text/plain; charset=utf-8")
        .body("Ok"))
}

/// readiness handler
#[get("/readiness")]
async fn readiness() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok()
        .content_type("text/plain; charset=utf-8")
        .body("Ok"))
}

/// hostname handler
#[get("/hostname")]
async fn hostname() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok()
        .content_type("text/plain; charset=utf-8")
        .body(gethostname().to_str().unwrap().to_owned()))
}


async fn p404() -> Result<fs::NamedFile> {
    Ok(fs::NamedFile::open("static/404.html")?.set_status_code(StatusCode::NOT_FOUND))
}

async fn enchiridion_response(
    req: HttpRequest,
    web::Path((chapter,)): web::Path<(usize,)>,
) -> Result<HttpResponse> {
    println!("{:?}", req);

    let e = texts::extract_enchiridion();
    if let Some(chapter_text) = e.get(chapter - 1) {
        Ok(HttpResponse::Ok()
            .content_type("text/plain; charset=utf-8")
            .body(*chapter_text))
    } else {
        let response = p404().await?;
        response.into_response(&req)
    }

}

#[derive(Debug, StructOpt)]
struct Opt {
  #[structopt(default_value="0.0.0.0:8080")]
  bind_address: String,
  secret_key: Option<String>,
  config_key: Option<String>,
  config_map_file: Option<PathBuf>,
  secret_map_file: Option<PathBuf>
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    env::set_var("RUST_LOG", "actix_web=debug,actix_server=info");
    env_logger::init();
    let opt = Opt::from_args();
  println!("{:?}", opt);

    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            // register favicon
            .service(favicon)
            .service(health)
            .service(liveness)
            .service(readiness)
            .service(hostname)
            .service(
                web::resource("/enchiridion/{chapter}").route(web::get().to(enchiridion_response)),
            )
            .default_service(
                web::resource("")
                    .route(web::get().to(p404))
                    // all requests that are not GET
                    .route(
                        web::route()
                            .guard(guard::Not(guard::Get()))
                            .to(HttpResponse::MethodNotAllowed),
                    ),
            )
    })
    .bind(opt.bind_address)?
    .run()
    .await
}
