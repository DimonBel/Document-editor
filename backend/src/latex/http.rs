use actix_web::{post, web, HttpResponse};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::process::Command;
use uuid::Uuid;
use super::docx_writer;

/// Maximum accepted LaTeX source size for compilation: 1 MiB.
/// Anything larger is rejected before we touch pdflatex or the filesystem.
const MAX_SOURCE_BYTES: usize = 1024 * 1024;

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
/// The frontend uses KaTeX (browser-side) by default and only calls this
/// endpoint when the user explicitly requests a server-side PDF export.
///
/// Requires pdflatex on PATH:
///   Linux  – sudo apt install texlive-latex-base
///   Windows – install MiKTeX from https://miktex.org
#[post("/api/latex/compile")]
pub async fn compile_latex(body: web::Json<CompileRequest>) -> HttpResponse {
    if body.source.len() > MAX_SOURCE_BYTES {
        return HttpResponse::PayloadTooLarge().json(CompileError {
            error: format!(
                "LaTeX source exceeds {} bytes ({}).",
                MAX_SOURCE_BYTES,
                body.source.len()
            ),
            log: String::new(),
        });
    }

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

    let output_dir = match tmp_dir.to_str() {
        Some(s) => s,
        None => {
            cleanup(&tmp_dir).await;
            return internal_err("Temp directory path is not valid UTF-8", "");
        }
    };
    let tex_arg = match tex_path.to_str() {
        Some(s) => s,
        None => {
            cleanup(&tmp_dir).await;
            return internal_err("TeX file path is not valid UTF-8", "");
        }
    };

    // Run pdflatex with `-no-shell-escape` so a malicious document
    // cannot invoke arbitrary commands via \write18 or \immediate.
    let output = Command::new("pdflatex")
        .args([
            "-interaction=nonstopmode",
            "-halt-on-error",
            "-no-shell-escape",
            "-output-directory",
            output_dir,
            tex_arg,
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
            // Combine stdout+stderr so the frontend can surface the
            // real LaTeX error context (file + line) instead of an
            // opaque "compilation failed".
            let mut log = String::from_utf8_lossy(&out.stdout).to_string();
            let stderr = String::from_utf8_lossy(&out.stderr);
            if !stderr.is_empty() {
                if !log.is_empty() && !log.ends_with('\n') {
                    log.push('\n');
                }
                log.push_str(&stderr);
            }
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

/// POST /api/latex/to-docx
/// Converts LaTeX source to DOCX using the built-in Rust converter.
/// No external tools required.
#[post("/api/latex/to-docx")]
pub async fn latex_to_docx(body: web::Json<CompileRequest>) -> HttpResponse {
    match docx_writer::build_docx(&body.source) {
        Ok(bytes) => HttpResponse::Ok()
            .content_type("application/vnd.openxmlformats-officedocument.wordprocessingml.document")
            .append_header(("Content-Disposition", "attachment; filename=\"document.docx\""))
            .body(bytes),
        Err(e) => HttpResponse::InternalServerError().json(CompileError {
            error: e,
            log: String::new(),
        }),
    }
}
