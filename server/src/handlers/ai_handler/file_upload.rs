//! File upload handler for AI context

use axum::{
    Json,
    extract::{Multipart, State},
};
use axum_extra::TypedHeader;
use axum_extra::headers::Authorization;
use axum_extra::headers::authorization::Bearer;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::core::AppState;
use crate::errors::{AppError, AppResult};

#[derive(Debug, Serialize, Deserialize)]
pub struct FileUpload {
    pub name: String,
    pub content: String,
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

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("Multipart error: {e}")))?
    {
        let name = field.name().unwrap_or("unknown").to_string();
        let data = field
            .bytes()
            .await
            .map_err(|e| AppError::BadRequest(format!("Failed to read file: {e}")))?;

        let content = String::from_utf8(data.to_vec())
            .map_err(|e| AppError::BadRequest(format!("Invalid UTF-8 content: {e}")))?;

        files.push(FileUpload { name, content });
    }

    Ok(Json(serde_json::json!({
        "files_uploaded": files.len(),
        "files": files
    })))
}
