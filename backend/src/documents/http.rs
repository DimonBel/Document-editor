use std::sync::{Arc, Mutex};

use actix_web::{get, post, web, HttpResponse};
use serde::Deserialize;

use crate::documents::manager::DocumentManager;

#[derive(Deserialize)]
pub struct CreateDocRequest {
    pub title: String,
}

#[derive(serde::Serialize)]
struct DocResponse {
    id: String,
    title: String,
    content: String,
    created_at: chrono::DateTime<chrono::Utc>,
    client_count: usize,
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
    let doc_id = path.into_inner();
    
    if let Some(info) = mgr.get_document(&doc_id) {
        let content = mgr.get_content(&doc_id).unwrap_or_default();
        HttpResponse::Ok().json(DocResponse {
            id: info.id,
            title: info.title,
            content,
            created_at: info.created_at,
            client_count: info.client_count,
        })
    } else {
        HttpResponse::NotFound()
            .json(serde_json::json!({ "error": "Document not found" }))
    }
}

#[get("/api/documents")]
pub async fn list_documents(
    docs: web::Data<Arc<Mutex<DocumentManager>>>,
) -> HttpResponse {
    let mgr = docs.lock().unwrap();
    HttpResponse::Ok().json(mgr.list_documents())
}