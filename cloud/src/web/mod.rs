use std::sync::{mpsc, Mutex};
use std::time::Duration;

use actix_cors::Cors;
use actix_web::dev::ServerHandle;
use actix_web::http::{header, StatusCode};
use actix_web::middleware::ErrorHandlers;
use actix_web::web::{scope, Data};
use actix_web::{middleware, App, HttpServer};
use log::info;
use crate::fs::dav::dav::DAV_PREFIX;

use crate::service::CONTEXT;
use crate::storage::storage_facade::StorageFacade;
use crate::web::common::add_error_header;

mod cloud;
mod common;
pub(crate) mod vo;
mod dav;

pub(crate) struct AppState {
    facade_cloud: Mutex<StorageFacade>, // <- Mutex is necessary to mutate safely across threads
}

pub async fn run_web(tx: mpsc::Sender<ServerHandle>) -> std::io::Result<()> {
    CONTEXT.init_pool().await;

    let server = HttpServer::new({
        move || {
            let cloud = StorageFacade::new();
            let state = Data::new(AppState {
                facade_cloud: Mutex::new(cloud),
            });
            let cors = Cors::default()
                // .allowed_origin("*")
                .allowed_origin_fn(|_origin, _req_head| true)
                .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
                .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
                .allowed_header(header::CONTENT_TYPE)
                .max_age(3600);
            App::new()
                .wrap(middleware::Logger::default())
                .app_data(state.clone())
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
                    .configure(dav::config)
                )
                .configure(cloud_ui::config)
        }
    })
    .keep_alive(Duration::from_secs(120))
    .bind(("0.0.0.0", 8088))
    .unwrap()
    .run();
    let _ = tx.send(server.handle());

    info!("start at http://{}:{}/", "0.0.0.0", 8088);
    server.await
}
