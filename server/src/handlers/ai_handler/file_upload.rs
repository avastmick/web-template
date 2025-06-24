//! File upload handler for AI context

use axum::{
    Json,
    extract::{Multipart, State},
};
use axum_extra::TypedHeader;
use axum_extra::headers::Authorization;
use axum_extra::headers::authorization::Bearer;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;

use crate::core::AppState;
use crate::errors::{AppError, AppResult};

const DEFAULT_MAX_TOKENS: usize = 10_000;
const AVG_CHARS_PER_TOKEN: usize = 4; // Rough estimate: 1 token ≈ 4 characters

/// Get the maximum token limit from environment variable or use default
fn get_max_tokens() -> usize {
    std::env::var("MAX_FILE_CONTEXT_TOKENS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(DEFAULT_MAX_TOKENS)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileUpload {
    pub name: String,
    pub content: String,
    pub mime_type: Option<String>,
    pub size: usize,
}

#[derive(Debug)]
pub struct RawFileUpload {
    pub name: String,
    pub data: Vec<u8>,
    pub mime_type: Option<String>,
}

/// Extract text from PDF files using lopdf
///
/// # Errors
///
/// Returns an error if PDF parsing or text extraction fails.
fn extract_pdf_text(data: &[u8]) -> AppResult<String> {
    use lopdf::Document;

    let doc = Document::load_mem(data)
        .map_err(|e| AppError::BadRequest(format!("Failed to parse PDF: {e}")))?;

    let mut text = String::new();
    let pages = doc.get_pages();

    for (page_num, _page_id) in pages {
        if let Ok(page_text) = doc.extract_text(&[page_num]) {
            text.push_str(&page_text);
            text.push('\n');
        }
    }

    if text.is_empty() {
        Err(AppError::BadRequest(
            "No text content found in PDF".to_string(),
        ))
    } else {
        Ok(text)
    }
}

/// Extract text from DOCX files using docx-rs
///
/// # Errors
///
/// Returns an error if DOCX parsing or text extraction fails.
fn extract_docx_text(data: &[u8]) -> AppResult<String> {
    use docx_rs::read_docx;

    let docx =
        read_docx(data).map_err(|e| AppError::BadRequest(format!("Failed to parse DOCX: {e}")))?;

    // Extract all text from paragraphs
    let mut text = String::new();
    for paragraph in &docx.document.children {
        if let docx_rs::DocumentChild::Paragraph(p) = paragraph {
            for child in &p.children {
                if let docx_rs::ParagraphChild::Run(run) = child {
                    for run_child in &run.children {
                        if let docx_rs::RunChild::Text(t) = run_child {
                            text.push_str(&t.text);
                        }
                    }
                }
            }
            text.push('\n');
        }
    }

    if text.trim().is_empty() {
        Err(AppError::BadRequest(
            "No text content found in DOCX".to_string(),
        ))
    } else {
        Ok(text)
    }
}

/// Extract text content from various file types
///
/// # Errors
///
/// Returns an error if text extraction fails.
fn extract_text_content(file: &RawFileUpload) -> AppResult<String> {
    // Determine file type from mime type or file extension
    let file_type = file.mime_type.as_deref().unwrap_or_else(|| {
        let path = Path::new(&file.name);
        if let Some(ext) = path.extension() {
            if ext.eq_ignore_ascii_case("pdf") {
                "application/pdf"
            } else if ext.eq_ignore_ascii_case("docx") {
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
            } else if ext.eq_ignore_ascii_case("txt") {
                "text/plain"
            } else {
                "application/octet-stream"
            }
        } else {
            "application/octet-stream"
        }
    });

    match file_type {
        "text/plain" | "text/html" | "text/markdown" | "text/csv" | "application/json" => {
            // For text files, try UTF-8 decoding
            String::from_utf8(file.data.clone())
                .map_err(|e| AppError::BadRequest(format!("Invalid UTF-8 in text file: {e}")))
        }
        "application/pdf" => extract_pdf_text(&file.data),
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document" => {
            extract_docx_text(&file.data)
        }
        _ => Err(AppError::BadRequest(format!(
            "Unsupported file type: {file_type}"
        ))),
    }
}

/// Estimate token count from character count
fn estimate_tokens(text: &str) -> usize {
    text.len() / AVG_CHARS_PER_TOKEN
}

/// Truncate text to fit within token limit
fn truncate_to_token_limit(text: String, max_tokens: usize) -> String {
    let estimated_tokens = estimate_tokens(&text);
    if estimated_tokens <= max_tokens {
        return text;
    }

    // Calculate approximate character limit
    let char_limit = max_tokens * AVG_CHARS_PER_TOKEN;
    let mut truncated = text.chars().take(char_limit).collect::<String>();

    // Add truncation notice
    truncated.push_str("\n\n[Content truncated to fit within token limit]");
    truncated
}

/// Handle file upload for chat context
///
/// # Errors
///
/// Returns an error if file upload fails or authentication is invalid.
pub async fn upload_file_handler(
    State(state): State<Arc<AppState>>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    mut multipart: Multipart,
) -> AppResult<Json<serde_json::Value>> {
    // Verify JWT token and get user
    let token = auth.token();
    let _user_id = state
        .auth_service
        .get_user_id_from_token(token)?
        .to_string();

    let mut files = Vec::new();
    let mut raw_files = Vec::new();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("Multipart error: {e}")))?
    {
        let file_name = field.file_name().unwrap_or("unknown").to_string();
        let content_type = field.content_type().map(std::string::ToString::to_string);

        let data = field
            .bytes()
            .await
            .map_err(|e| AppError::BadRequest(format!("Failed to read file: {e}")))?;

        raw_files.push(RawFileUpload {
            name: file_name.clone(),
            data: data.to_vec(),
            mime_type: content_type,
        });
    }

    // Process each file and extract text content with token limit
    let mut total_tokens = 0;
    let mut truncated_files = 0;
    let max_tokens = get_max_tokens();

    for raw_file in &raw_files {
        let mut content = extract_text_content(raw_file)?;
        let file_tokens = estimate_tokens(&content);

        // Check if adding this file would exceed the limit
        if total_tokens + file_tokens > max_tokens {
            let remaining_tokens = max_tokens.saturating_sub(total_tokens);
            if remaining_tokens > 0 {
                // Truncate this file to fit within remaining tokens
                content = truncate_to_token_limit(content, remaining_tokens);
                truncated_files += 1;
            } else {
                // Skip this file entirely
                continue;
            }
        }

        total_tokens += estimate_tokens(&content);
        let size = raw_file.data.len();

        files.push(FileUpload {
            name: raw_file.name.clone(),
            content,
            mime_type: raw_file.mime_type.clone(),
            size,
        });

        // Stop processing if we've reached the token limit
        if total_tokens >= max_tokens {
            break;
        }
    }

    let mut response = serde_json::json!({
        "files_uploaded": files.len(),
        "files": files,
        "total_estimated_tokens": total_tokens,
        "max_tokens": max_tokens
    });

    if truncated_files > 0 {
        response["truncated_files"] = serde_json::json!(truncated_files);
    }

    if raw_files.len() > files.len() {
        response["skipped_files"] = serde_json::json!(raw_files.len() - files.len());
    }

    Ok(Json(response))
}
