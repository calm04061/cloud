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

use api::{Capacity, PluginInstance, PluginMetaInfo};
use service::CONTEXT;

use crate::fs::dav::cloud_dav_filesystem::DAV_PREFIX;
use crate::web::common::add_error_header;

mod cloud;
mod common;
pub(crate) mod vo;
mod dav;
mod rate_limit;
mod health;
mod metrics;
use actix_web::dev::Service;

async fn validator(
    req: ServiceRequest,
    credentials: BasicAuth,
) -> Result<ServiceRequest, (actix_web::Error, ServiceRequest)> {
    let rate_limit_key = auth_rate_limit_key(&req, &credentials);
    if !rate_limit::allowed(&rate_limit_key) {
        let my_realm = "Dav Realm".to_string();
        let challenge = Basic::with_realm(my_realm);
        return Err((AuthenticationError::new(challenge).into(), req));
    }
    let auth = validate(credentials).await;
    if !auth {
        rate_limit::failure(&rate_limit_key);
        let my_realm = "Dav Realm".to_string();
        let challenge = Basic::with_realm(my_realm);
        return Err((AuthenticationError::new(challenge).into(), req));
    }
    rate_limit::success(&rate_limit_key);
    Ok(req)
}

fn auth_rate_limit_key(req: &ServiceRequest, credentials: &BasicAuth) -> String {
    let ip = req
        .peer_addr()
        .map(|addr| addr.ip().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    format!("{}@{}", credentials.user_id(), ip)
}

async fn validate(credentials: BasicAuth) -> bool {
    if credentials.password().is_none() {
        return false;
    }
    let username = credentials.user_id();
    let result = match CONTEXT.user_manager.select_by_username(username).await {
        Ok(r) => r,
        Err(_) => return false,
    };
    if result.is_none() {
        return false;
    }
    let password = credentials.password().unwrap();
    let user = result.unwrap();
    match persistence::utils::verify_password(password, &user.password) {
        Ok(valid) => valid,
        Err(_) => false,
    }
}

pub async fn run_web(plugin_arc: Arc<Vec<PluginMetaInfo>>) -> Server {
    let port = dotenvy::var("HTTP_PORT")
        .unwrap_or("8088".into())
        .parse::<u16>()
        .unwrap_or(8088);
    let cache_file = crate::fs::config::temp_path().await;
    let server = HttpServer::new({
        move || {
            let basic_auth = HttpAuthentication::basic(validator);
            let allowed_origins: Vec<String> = dotenvy::var("ALLOWED_ORIGINS")
                .unwrap_or_else(|_| "http://localhost:5173".to_string())
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
            let cors = Cors::default()
                .allowed_origin_fn(move |origin, _req_head| {
                    let origin_str = match origin.to_str() {
                        Ok(s) => s,
                        Err(_) => return false,
                    };
                    allowed_origins.iter().any(|o| o == origin_str)
                })
                .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
                .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
                .allowed_header(header::CONTENT_TYPE)
                .max_age(3600);
            let mut app = App::new()
                .wrap(middleware::Logger::default())
                .wrap(cors)
                // 探针端点无需认证，供负载均衡/K8s 探测
                .service(health::health)
                .service(health::ready)
                .service(metrics::metrics)
                .service(
                    scope("api")
                        .wrap(basic_auth.clone())
                        .wrap(
                            ErrorHandlers::new()
                                .handler(StatusCode::INTERNAL_SERVER_ERROR, add_error_header),
                        )
                        .configure(cloud::config),
                )
                .service(
                    scope(DAV_PREFIX)
                        .wrap_fn(|mut req, srv|{
                            let headers = req.headers_mut();
                            headers.remove("if-modified-since");
                            let fut = srv. call(req);
                            async {
                                let res = fut.await?;
                                Ok(res)
                            }
                        })
                        .wrap(basic_auth)
                        .configure(|cfg| dav::config(cfg, cache_file.as_str()))
                );

            for p in plugin_arc.iter() {
                let meta_info = &p.meta_info;
                let cas = meta_info.capacities.iter().clone();
                for c in cas {
                    match c {
                        Capacity::WEB(name) => {
                            match &p.instance {
                                PluginInstance::Native(library) => unsafe {
                                    let web: Result<Symbol<fn(cfg: &mut actix_web::web::ServiceConfig)>, Error> = library.get(name.as_bytes());
                                    if web.is_ok() {
                                        let web = web.unwrap();
                                        app = app.configure(|cfg| {
                                            web(cfg);
                                        });
                                    }
                                },
                                PluginInstance::Wasi(_) => {
                                    info!("WASI plugin {} does not support WEB capacity yet", meta_info.name);
                                }
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
