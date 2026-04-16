use std::sync::{Arc, Mutex};

use actix_web::{get, post, web, HttpResponse};
use serde::Deserialize;

use crate::documents::manager::DocumentManager;

#[derive(Deserialize)]
pub struct CreateDocRequest {
    pub title: String,
}

#[post("/api/documents")]
pub async fn create_document(
    body: web::Json<CreateDocRequest>,
    docs: web::Data<Arc<Mutex<DocumentManager>>>,
) -> HttpResponse {
    let mut mgr = docs.lock().unwrap();
    let doc = mgr.create_document(body.title.clone());
    HttpResponse::Ok().json(doc)
}

#[get("/api/documents/{doc_id}")]
pub async fn get_document(
    path: web::Path<String>,
    docs: web::Data<Arc<Mutex<DocumentManager>>>,
) -> HttpResponse {
    let mgr = docs.lock().unwrap();
    match mgr.get_document(&path.into_inner()) {
        Some(doc) => HttpResponse::Ok().json(doc),
        None => HttpResponse::NotFound()
            .json(serde_json::json!({ "error": "Document not found" })),
    }
}

#[get("/api/documents")]
pub async fn list_documents(
    docs: web::Data<Arc<Mutex<DocumentManager>>>,
) -> HttpResponse {
    let mgr = docs.lock().unwrap();
    HttpResponse::Ok().json(mgr.list_documents())
}