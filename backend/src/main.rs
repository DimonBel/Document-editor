mod broadcast;
mod config;
mod crdt;
mod documents;
mod handlers;
mod models;
mod rooms;

use actix_cors::Cors;
use actix_web::{middleware, web, App, HttpServer};
use documents::manager::DocumentManager;
use rooms::manager::RoomManager;
use std::sync::{Arc, Mutex};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let cfg = config::Config::from_env();
    let room_manager = web::Data::new(Arc::new(Mutex::new(RoomManager::new())));
    let doc_manager = web::Data::new(Arc::new(Mutex::new(DocumentManager::new())));

    log::info!("Starting server at {}:{}", cfg.host, cfg.port);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .app_data(room_manager.clone())
            .app_data(doc_manager.clone())
            .wrap(cors)
            .wrap(middleware::Logger::default())
            .service(handlers::http::create_room)
            .service(handlers::http::get_room)
            .service(handlers::http::list_rooms)
            .service(handlers::websocket::ws_route)
            .service(documents::http::create_document)
            .service(documents::http::get_document)
            .service(documents::http::list_documents)
            .service(documents::ws::doc_ws_route)
    })
    .bind(format!("{}:{}", cfg.host, cfg.port))?
    .run()
    .await
}
