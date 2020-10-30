#![feature(proc_macro_hygiene)]

use actix_files as fs;
use actix_web::http::{header, Method, StatusCode};
use actix_web::{
    error, get, guard, middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer, Result,
};
use lazy_static::lazy_static;
use maud::html;
use std::{env, io};

mod texts;

/// favicon handler
#[get("/favicon.ico")]
async fn favicon() -> Result<fs::NamedFile> {
    Ok(fs::NamedFile::open("static/favicon.ico")?)
}

fn get_p404() -> &'static str {
    lazy_static! {
        static ref P404: String = html! {
            p { "No results found" }
        }
        .into_string();
    }
    &P404
}

async fn p404() -> Result<HttpResponse> {
    Ok(HttpResponse::build(StatusCode::NOT_FOUND)
        .content_type("text/html; charset=utf-8")
        .body(get_p404()))
}

async fn enchiridion_response(
    req: HttpRequest,
    web::Path((chapter,)): web::Path<(usize,)>,
) -> HttpResponse {
    println!("{:?}", req);

    let e = texts::extract_enchiridion();
    if let Some(chapter_text) = e.get(chapter - 1) {
        HttpResponse::Ok()
            .content_type("text/plain")
            .body(*chapter_text)
    } else {
        HttpResponse::build(StatusCode::NOT_FOUND)
            .content_type("text/html; charset=utf-8")
            .body(get_p404())
    }
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    env::set_var("RUST_LOG", "actix_web=debug,actix_server=info");
    env_logger::init();

    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            // register favicon
            .service(favicon)
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
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
