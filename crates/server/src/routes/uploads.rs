use std::sync::Arc;

use axum::{
    Json,
    body::Body,
    extract::{Multipart, Path, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{error::ApiError, extract::AuthUser, state::AppState};

const MAX_FILE_SIZE: usize = 10 * 1024 * 1024; // 10 MB for all files
const MAX_MULTIPART_SIZE: usize = 10 * 1024 * 1024; // 10 MB for file uploads

// ---- Base64 image upload (simple JSON endpoint) ----

#[derive(Deserialize)]
pub struct Base64UploadRequest {
    pub filename: String,
    pub content_type: String,
    pub data: String, // base64-encoded image data
}

/// POST /channels/{channel_id}/files — upload any file as base64, attach to a new message
pub async fn upload_file_base64(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(channel_id): Path<Uuid>,
    Json(body): Json<Base64UploadRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Verify channel access
    let server_id = rusteze_db::members::channel_server_id(&state.db, channel_id)
        .await?;

    match server_id {
        Some(sid) => {
            if !rusteze_db::members::is_member(&state.db, sid, user.0).await? {
                return Err(ApiError {
                    status: StatusCode::FORBIDDEN,
                    message: "not a member of this server".into(),
                });
            }
        }
        None => {
            if !rusteze_db::dms::is_dm_member(&state.db, channel_id, user.0).await? {
                return Err(ApiError {
                    status: StatusCode::FORBIDDEN,
                    message: "not a participant".into(),
                });
            }
        }
    }

    // Validate size (base64 is ~33% larger than raw)
    let raw_size = body.data.len() * 3 / 4;
    if raw_size > MAX_FILE_SIZE {
        return Err(ApiError {
            status: StatusCode::PAYLOAD_TOO_LARGE,
            message: format!("image exceeds {}MB limit", MAX_FILE_SIZE / 1024 / 1024),
        });
    }

    // Create a message with no text content (image-only message)
    let msg = rusteze_db::messages::create_message(
        &state.db, channel_id, user.0, None, None,
    ).await?;

    // Store the base64 data directly in the database
    let att = rusteze_db::attachments::create_attachment_base64(
        &state.db,
        msg.id,
        &body.filename,
        &body.content_type,
        raw_size as i64,
        &body.data,
    ).await?;

    // Broadcast MessageCreate with attachment info
    let event = rusteze_models::ServerEvent::MessageCreate(rusteze_models::Message {
        id: msg.id,
        channel_id: msg.channel_id,
        author_id: msg.author_id,
        content: msg.content.clone(),
        attachments: vec![rusteze_models::Attachment {
            id: att.id,
            filename: att.filename.clone(),
            content_type: att.content_type.clone(),
            size: att.size as u64,
            url: format!("/attachments/{}", att.id),
        }],
        embeds: vec![],
        mentions: vec![],
        replies_to: msg.replies_to,
        pinned: msg.pinned,
        edited_at: msg.edited_at,
        created_at: msg.created_at,
    });

    if let Ok(payload) = serde_json::to_string(&event) {
        let _: Result<(), _> = fred::interfaces::PubsubInterface::publish(
            &state.redis,
            format!("channel:{channel_id}"),
            payload.as_str(),
        )
        .await;
    }

    Ok(Json(serde_json::json!({
        "message_id": msg.id,
        "attachment_id": att.id,
        "url": format!("/attachments/{}", att.id),
    })))
}

// ---- Download / serve attachment ----

/// GET /attachments/{attachment_id} — serve a file (base64 decoded or from disk)
pub async fn get_attachment(
    State(state): State<Arc<AppState>>,
    Path(attachment_id): Path<Uuid>,
) -> Result<Response, ApiError> {
    let att = rusteze_db::attachments::fetch_attachment(&state.db, attachment_id).await?;

    // If we have base64 data in DB, decode and serve it
    if let Some(ref base64_data) = att.data {
        use base64::Engine;
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(base64_data)
            .map_err(|_| ApiError {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                message: "corrupt base64 data".into(),
            })?;

        let response = Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, &att.content_type)
            .header(header::CONTENT_LENGTH, decoded.len())
            .header(
                header::CONTENT_DISPOSITION,
                format!("inline; filename=\"{}\"", att.filename),
            )
            .header(header::CACHE_CONTROL, "public, max-age=31536000")
            .body(Body::from(decoded))
            .unwrap();

        return Ok(response);
    }

    // Fallback: serve from filesystem
    let data = state
        .media
        .fetch(&att.storage_path)
        .await
        .map_err(|_| ApiError {
            status: StatusCode::NOT_FOUND,
            message: "file not found".into(),
        })?;

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, &att.content_type)
        .header(
            header::CONTENT_DISPOSITION,
            format!("inline; filename=\"{}\"", att.filename),
        )
        .header(header::CONTENT_LENGTH, data.len())
        .body(Body::from(data))
        .unwrap();

    Ok(response)
}

/// GET /attachments/{attachment_id}/base64 — return the raw base64 string (for client rendering)
pub async fn get_attachment_base64(
    State(state): State<Arc<AppState>>,
    Path(attachment_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let att = rusteze_db::attachments::fetch_attachment(&state.db, attachment_id).await?;

    Ok(Json(serde_json::json!({
        "id": att.id,
        "filename": att.filename,
        "content_type": att.content_type,
        "size": att.size,
        "data": att.data,
    })))
}

// ---- Multipart file upload (legacy, for large files) ----

/// POST /channels/{channel_id}/upload — multipart file upload
pub async fn upload_file(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(channel_id): Path<Uuid>,
    mut multipart: Multipart,
) -> Result<Json<Vec<rusteze_db::attachments::AttachmentRow>>, ApiError> {
    let server_id = rusteze_db::members::channel_server_id(&state.db, channel_id)
        .await?
        .ok_or(ApiError {
            status: StatusCode::NOT_FOUND,
            message: "channel not found".into(),
        })?;

    if !rusteze_db::members::is_member(&state.db, server_id, user.0).await? {
        return Err(ApiError {
            status: StatusCode::FORBIDDEN,
            message: "not a member of this server".into(),
        });
    }

    let msg = rusteze_db::messages::create_message(&state.db, channel_id, user.0, None, None).await?;
    let mut attachments = Vec::new();

    while let Some(field) = multipart.next_field().await.map_err(|e| ApiError {
        status: StatusCode::BAD_REQUEST,
        message: format!("multipart error: {e}"),
    })? {
        let filename = field.file_name().unwrap_or("upload.bin").to_string();
        let content_type = field.content_type().unwrap_or("application/octet-stream").to_string();

        let data = field.bytes().await.map_err(|e| ApiError {
            status: StatusCode::BAD_REQUEST,
            message: format!("read error: {e}"),
        })?;

        if data.len() > MAX_MULTIPART_SIZE {
            return Err(ApiError {
                status: StatusCode::PAYLOAD_TOO_LARGE,
                message: format!("file exceeds {}MB limit", MAX_MULTIPART_SIZE / 1024 / 1024),
            });
        }

        let storage_path = state.media.store(&data, &filename).await.map_err(|e| ApiError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: format!("storage error: {e}"),
        })?;

        let att = rusteze_db::attachments::create_attachment(
            &state.db, msg.id, &filename, &content_type, data.len() as i64, &storage_path,
        ).await?;

        attachments.push(att);
    }

    Ok(Json(attachments))
}
