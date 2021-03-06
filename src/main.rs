use actix_files as fs;
use actix_web::http::{StatusCode};
use actix_web::{
    get, guard, middleware, web, App, HttpRequest, HttpResponse, HttpServer, Result,
};
use gethostname::gethostname;
use lazy_static::lazy_static;
use std::path::{PathBuf, Path};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::{env, io};
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

fn resource_404() -> Result<fs::NamedFile> {
    Ok(fs::NamedFile::open("static/404.html")?.set_status_code(StatusCode::NOT_FOUND))
}

async fn async_404() -> Result<fs::NamedFile> {
    resource_404()
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
        let response = resource_404()?;
        response.into_response(&req)
    }
}

#[derive(Debug, Clone, StructOpt)]
struct Opt {
    #[structopt(short, long, env = "ST_BIND_ADDRESS", default_value = "0.0.0.0:8080")]
    bind_address: String,
    #[structopt(long, env = "ST_SECRET_KEY")]
    secret_key: Option<String>,
    #[structopt(long, env = "ST_CONFIG_KEY")]
    config_key: Option<String>,
    #[structopt(long, env = "ST_CONFIG_FILE")]
    config_file: Option<PathBuf>,
    #[structopt(long, env = "ST_CONFIG_MAP_FILE")]
    config_map_file: Option<PathBuf>,
    #[structopt(long, env = "ST_SECRET_MAP_FILE")]
    secret_map_file: Option<PathBuf>,
}

impl Opt {
    fn str_response(option: &Option<String>) -> Result<HttpResponse> {
        if let Some(s) = &option {
            Ok(HttpResponse::Ok()
                .content_type("text/plain; charset=utf-8")
                .body(s))
        } else {
            Ok(HttpResponse::NotFound()
                .content_type("text/plain; charset=utf-8")
                .body("Not Configured"))
        }
    }

    fn file_response<T: AsRef<Path>>(option: &Option<T>) -> Result<fs::NamedFile> {
        if let Some(p) = option {
            Ok(fs::NamedFile::open(p.as_ref())?)
        } else {
            resource_404()
        }
    }
}

async fn secret_key(data: web::Data<Opt>) -> Result<HttpResponse> {
    Opt::str_response(&data.secret_key)
}

async fn config_key(data: web::Data<Opt>) -> Result<HttpResponse> {
    Opt::str_response(&data.config_key)
}

async fn config_file(data: web::Data<Opt>) -> Result<fs::NamedFile> {
    Opt::file_response(&data.config_file)
}

async fn config_map_file(data: web::Data<Opt>) -> Result<fs::NamedFile> {
    Opt::file_response(&data.config_map_file)
}

async fn secret_map_file(data: web::Data<Opt>) -> Result<fs::NamedFile> {
    Opt::file_response(&data.secret_map_file)
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    env::set_var("RUST_LOG", "actix_web=debug,actix_server=info");
    env_logger::init();
    let opt = Opt::from_args();
    let data = opt.clone();
    println!("{:?}", opt);

    if let Some(config_file) = &opt.config_file {
        if !config_file.as_path().exists() {
            std::fs::copy("static/enchiridion.txt", config_file)?;
        }
    }

    HttpServer::new(move || {
        App::new()
            .data(data.clone())
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
            .service(web::resource("/config").route(web::get().to(config_key)))
            .service(web::resource("/secret").route(web::get().to(secret_key)))
            .service(web::resource("/config_file").route(web::get().to(config_file)))
            .service(web::resource("/config_map_file").route(web::get().to(config_map_file)))
            .service(web::resource("/config_secret_file").route(web::get().to(secret_map_file)))
            .default_service(
                web::resource("")
                    .route(web::get().to(async_404))
                    // all requests that are not GET
                    .route(
                        web::route()
                            .guard(guard::Not(guard::Get()))
                            .to(HttpResponse::MethodNotAllowed),
                    ),
            )
    })
    .bind(&opt.bind_address)?
    .run()
    .await

}
