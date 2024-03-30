use std::sync::Arc;
use std::time::Duration;

use actix_cors::Cors;
use actix_web::dev::{Server, ServiceRequest};
use actix_web::http::{header, StatusCode};
use actix_web::middleware::ErrorHandlers;
use actix_web::web::scope;
use actix_web::{middleware, App, HttpServer};
use actix_web_httpauth::extractors::basic::BasicAuth;
use actix_web_httpauth::extractors::AuthenticationError;
use actix_web_httpauth::headers::www_authenticate::basic::Basic;
use actix_web_httpauth::middleware::HttpAuthentication;
use libloading::{Error, Symbol};
use log::info;

use api::{Capacity, PluginMetaInfo};
use service::CONTEXT;

use crate::fs::dav::cluod_dav_filesystem::DAV_PREFIX;
use crate::web::common::add_error_header;

mod cloud;
mod common;
pub(crate) mod vo;
mod dav;

async fn validator(
    req: ServiceRequest,
    credentials: BasicAuth,
) -> Result<ServiceRequest, (actix_web::Error, ServiceRequest)> {
    let auth = validate(credentials).await;
    if !auth {
        let my_realm = "Dav Realm".to_string();
        let challenge = Basic::with_realm(my_realm);
        return Err((AuthenticationError::new(challenge).into(), req));
    }
    Ok(req)
}

async fn validate(credentials: BasicAuth) -> bool {
    if credentials.password().is_none() {
        return false;
    }
    let username = credentials.user_id();
    let result = CONTEXT.user_manager.select_by_username(username).await.unwrap();
    if result.is_none() {
        return false;
    }
    let password = credentials.password().unwrap();
    let user = result.unwrap();
    password.eq(&user.password)
}

pub async fn run_web(plugin_arc: Arc<Vec<PluginMetaInfo>>) -> Server {
    CONTEXT.init_pool().await;
    CONTEXT.upgrade().await;
    let port = dotenvy::var("HTTP_PORT")
        .unwrap_or("8088".to_string())
        .parse::<u16>()
        .unwrap_or(8088);
    let server = HttpServer::new({
        move || {
            let auth = HttpAuthentication::basic(validator);
            let cors = Cors::default()
                // .allowed_origin("*")
                .allowed_origin_fn(|_origin, _req_head| true)
                .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
                .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
                .allowed_header(header::CONTENT_TYPE)
                .max_age(3600);
            let mut app = App::new()
                .wrap(middleware::Logger::default())
                .wrap(cors)
                .service(
                    scope("api")
                        .wrap(
                            ErrorHandlers::new()
                                .handler(StatusCode::INTERNAL_SERVER_ERROR, add_error_header),
                        )
                        .configure(cloud::config),
                )
                .service(
                    scope(DAV_PREFIX)
                        .wrap(auth)
                        .configure(dav::config)
                );

            for p in plugin_arc.iter() {
                let meta_info = &p.meta_info;
                let cas = meta_info.capacities.iter().clone();
                for c in cas {
                    match c {
                        Capacity::WEB(name) => unsafe {
                            let web: Result<Symbol<fn(cfg: &mut actix_web::web::ServiceConfig)>, Error> = p.library.get(name.as_bytes());
                            if web.is_ok() {
                                let web = web.unwrap();
                                app = app.configure(|cfg| {
                                    web(cfg);
                                });
                            }
                        }
                        _ => {}
                    }
                }
            }
            app
        }
    })
        .keep_alive(Duration::from_secs(120))
        .bind(("0.0.0.0", port))
        .unwrap()
        // .bind(("::", port))
        // .unwrap()
        .run();
    // let _ = tx.send(server.handle());

    info!("start at http://0.0.0.0:{port}");
    info!("start at http://[::]:{port}");
    server
}
