use std::sync::{Arc};
use std::time::Duration;

use actix_cors::Cors;
use actix_web::{App, HttpServer, middleware};
use actix_web::dev::{Server};
use actix_web::http::{header, StatusCode};
use actix_web::middleware::ErrorHandlers;
use actix_web::web::{Data, scope};
use libloading::{Error, Symbol};
use log::info;
use tokio::sync::Mutex;
use api::Capacity;

use crate::fs::dav::dav::DAV_PREFIX;
use crate::plugin::PluginMetaInfo;
use crate::service::CONTEXT;
use crate::storage::storage_facade::StorageFacade;
use crate::web::common::add_error_header;

mod cloud;
mod common;
pub(crate) mod vo;
mod dav;

pub(crate) struct AppState {
    facade_cloud: Arc<Mutex<StorageFacade>>, // <- Mutex is necessary to mutate safely across threads
}

pub async fn run_web(plugin_arc: Arc<Vec<PluginMetaInfo>>, arc: Arc<Mutex<StorageFacade>>) -> Server {
    CONTEXT.init_pool().await;
    CONTEXT.upgrade().await;
    let port = dotenvy::var("HTTP_PORT")
        .unwrap_or("8088".to_string())
        .parse::<u16>()
        .unwrap_or(8088);
    let server = HttpServer::new({
        move || {
            let state = Data::new(AppState {
                facade_cloud: Arc::clone(&arc),
            });
            let cors = Cors::default()
                // .allowed_origin("*")
                .allowed_origin_fn(|_origin, _req_head| true)
                .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
                .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
                .allowed_header(header::CONTENT_TYPE)
                .max_age(3600);
            let mut app = App::new()
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
                );
            // .configure(cloud_ui::config)

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

    info!("start at http://{}:{}", "0.0.0.0", port);
    info!("start at http://[{}]:{}", "::", port);
    server
}
