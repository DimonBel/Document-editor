use std::sync::{Arc, Mutex};

use actix_web::{get, post, web, HttpResponse};
use serde::Deserialize;

use crate::rooms::manager::RoomManager;

#[derive(Deserialize)]
pub struct CreateRoomRequest {
    pub name: String,
}

#[post("/api/rooms")]
pub async fn create_room(
    body: web::Json<CreateRoomRequest>,
    rooms: web::Data<Arc<Mutex<RoomManager>>>,
) -> HttpResponse {
    let mut mgr = rooms.lock().unwrap();
    let room = mgr.create_room(body.name.clone());
    HttpResponse::Ok().json(room)
}

#[get("/api/rooms/{room_id}")]
pub async fn get_room(
    path: web::Path<String>,
    rooms: web::Data<Arc<Mutex<RoomManager>>>,
) -> HttpResponse {
    let mgr = rooms.lock().unwrap();
    match mgr.get_room_info(&path.into_inner()) {
        Some(room) => HttpResponse::Ok().json(room),
        None => HttpResponse::NotFound()
            .json(serde_json::json!({ "error": "Room not found" })),
    }
}

#[get("/api/rooms")]
pub async fn list_rooms(
    rooms: web::Data<Arc<Mutex<RoomManager>>>,
) -> HttpResponse {
    let mgr = rooms.lock().unwrap();
    HttpResponse::Ok().json(mgr.list_rooms())
}
