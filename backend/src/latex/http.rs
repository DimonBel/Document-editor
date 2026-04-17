use actix_web::{post, web, HttpResponse};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::process::Command;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CompileRequest {
    pub source: String,
}

#[derive(Serialize)]
struct CompileError {
    error: String,
    log: String,
}

/// POST /api/latex/compile
///
/// Optional server-side PDF compilation using pdflatex.
/// The frontend uses latex.js (browser-side) by default and only calls this
/// endpoint when the user explicitly requests a PDF export.
///
/// Requires pdflatex on PATH:
///   Linux  – sudo apt install texlive-latex-base
///   Windows – install MiKTeX from https://miktex.org
#[post("/api/latex/compile")]
pub async fn compile_latex(body: web::Json<CompileRequest>) -> HttpResponse {
    let job_id = Uuid::new_v4().to_string();
    let tmp_dir = std::env::temp_dir().join(format!("latex_{}", job_id));

    if let Err(e) = tokio::fs::create_dir_all(&tmp_dir).await {
        return internal_err("Failed to create temp directory", &e.to_string());
    }

    let tex_path = tmp_dir.join("main.tex");
    if let Err(e) = tokio::fs::write(&tex_path, body.source.as_bytes()).await {
        cleanup(&tmp_dir).await;
        return internal_err("Failed to write source file", &e.to_string());
    }

    let output = Command::new("pdflatex")
        .args([
            "-interaction=nonstopmode",
            "-halt-on-error",
            "-output-directory",
            tmp_dir.to_str().unwrap_or("."),
            tex_path.to_str().unwrap_or("main.tex"),
        ])
        .output()
        .await;

    match output {
        Err(e) => {
            cleanup(&tmp_dir).await;
            HttpResponse::UnprocessableEntity().json(CompileError {
                error: format!(
                    "pdflatex not found. Install MiKTeX (Windows) or texlive (Linux).\nError: {}",
                    e
                ),
                log: String::new(),
            })
        }
        Ok(out) => {
            let log = String::from_utf8_lossy(&out.stdout).to_string();
            let pdf_path = tmp_dir.join("main.pdf");
            match tokio::fs::read(&pdf_path).await {
                Ok(bytes) => {
                    cleanup(&tmp_dir).await;
                    HttpResponse::Ok()
                        .content_type("application/pdf")
                        .body(bytes)
                }
                Err(_) => {
                    cleanup(&tmp_dir).await;
                    HttpResponse::UnprocessableEntity().json(CompileError {
                        error: "Compilation failed — no PDF produced.".to_string(),
                        log,
                    })
                }
            }
        }
    }
}

async fn cleanup(dir: &PathBuf) {
    let _ = tokio::fs::remove_dir_all(dir).await;
}

fn internal_err(msg: &str, detail: &str) -> HttpResponse {
    HttpResponse::InternalServerError().json(CompileError {
        error: format!("{}: {}", msg, detail),
        log: String::new(),
    })
}
