use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---- Response types (match server JSON) ----

#[derive(Debug, Deserialize)]
pub struct AuthResponse {
    pub user_id: Uuid,
    pub token: String,
}

#[derive(Debug, Deserialize)]
pub struct ServerRow {
    pub id: Uuid,
    pub name: String,
    pub owner_id: Uuid,
    pub icon_url: Option<String>,
    pub banner_url: Option<String>,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct ChannelRow {
    pub id: Uuid,
    pub server_id: Option<Uuid>,
    pub name: String,
    pub channel_type: String,
    pub topic: Option<String>,
    pub position: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct MessageRow {
    pub id: Uuid,
    pub channel_id: Uuid,
    pub author_id: Uuid,
    pub content: Option<String>,
    pub replies_to: Option<Uuid>,
    pub pinned: bool,
    pub edited_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct EditedMessageRow {
    pub id: Uuid,
    pub channel_id: Uuid,
    pub author_id: Uuid,
    pub content: Option<String>,
    pub edited_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

// ---- Public API (all blocking — call from background executor) ----

pub fn login(api_url: &str, email: &str, password: &str) -> Result<AuthResponse> {
    #[derive(Serialize)]
    struct Body<'a> {
        email: &'a str,
        password: &'a str,
    }
    let url = format!("{api_url}/auth/login");
    let body = Body { email, password };
    let resp: AuthResponse = ureq::post(&url)
        .header("Content-Type", "application/json")
        .send_json(&body)?
        .body_mut()
        .read_json()?;
    Ok(resp)
}

pub fn register(
    api_url: &str,
    username: &str,
    email: &str,
    password: &str,
) -> Result<AuthResponse> {
    #[derive(Serialize)]
    struct Body<'a> {
        username: &'a str,
        email: &'a str,
        password: &'a str,
    }
    let url = format!("{api_url}/auth/register");
    let body = Body { username, email, password };
    let resp: AuthResponse = ureq::post(&url)
        .header("Content-Type", "application/json")
        .send_json(&body)?
        .body_mut()
        .read_json()?;
    Ok(resp)
}

pub fn fetch_messages(
    api_url: &str,
    token: &str,
    channel_id: Uuid,
    before: Option<Uuid>,
) -> Result<Vec<MessageRow>> {
    let mut url = format!("{api_url}/channels/{channel_id}/messages?limit=50");
    if let Some(b) = before {
        url.push_str(&format!("&before={b}"));
    }
    let resp: Vec<MessageRow> = ureq::get(&url)
        .header("Authorization", &format!("Bearer {token}"))
        .call()?
        .body_mut()
        .read_json()?;
    Ok(resp)
}

pub fn send_message(
    api_url: &str,
    token: &str,
    channel_id: Uuid,
    content: &str,
) -> Result<MessageRow> {
    #[derive(Serialize)]
    struct Body<'a> {
        content: &'a str,
    }
    let url = format!("{api_url}/channels/{channel_id}/messages");
    let resp: MessageRow = ureq::post(&url)
        .header("Authorization", &format!("Bearer {token}"))
        .header("Content-Type", "application/json")
        .send_json(&Body { content })?
        .body_mut()
        .read_json()?;
    Ok(resp)
}

pub fn create_server(api_url: &str, token: &str, name: &str) -> Result<ServerRow> {
    #[derive(Serialize)]
    struct Body<'a> {
        name: &'a str,
    }
    let url = format!("{api_url}/servers");
    let resp: ServerRow = ureq::post(&url)
        .header("Authorization", &format!("Bearer {token}"))
        .header("Content-Type", "application/json")
        .send_json(&Body { name })?
        .body_mut()
        .read_json()?;
    Ok(resp)
}

pub fn fetch_servers(api_url: &str, token: &str) -> Result<Vec<ServerRow>> {
    let url = format!("{api_url}/servers");
    let resp: Vec<ServerRow> = ureq::get(&url)
        .header("Authorization", &format!("Bearer {token}"))
        .call()?
        .body_mut()
        .read_json()?;
    Ok(resp)
}

pub fn fetch_channels(api_url: &str, token: &str, server_id: Uuid) -> Result<Vec<ChannelRow>> {
    let url = format!("{api_url}/servers/{server_id}/channels");
    let resp: Vec<ChannelRow> = ureq::get(&url)
        .header("Authorization", &format!("Bearer {token}"))
        .call()?
        .body_mut()
        .read_json()?;
    Ok(resp)
}

pub fn create_channel(
    api_url: &str,
    token: &str,
    server_id: Uuid,
    name: &str,
    channel_type: &str,
) -> Result<ChannelRow> {
    #[derive(Serialize)]
    struct Body<'a> {
        name: &'a str,
        channel_type: &'a str,
    }
    let url = format!("{api_url}/servers/{server_id}/channels");
    let resp: ChannelRow = ureq::post(&url)
        .header("Authorization", &format!("Bearer {token}"))
        .header("Content-Type", "application/json")
        .send_json(&Body { name, channel_type })?
        .body_mut()
        .read_json()?;
    Ok(resp)
}

pub fn edit_message(
    api_url: &str,
    token: &str,
    channel_id: Uuid,
    message_id: Uuid,
    content: &str,
) -> Result<MessageRow> {
    #[derive(Serialize)]
    struct Body<'a> {
        content: &'a str,
    }
    let url = format!("{api_url}/channels/{channel_id}/messages/{message_id}");
    let resp: MessageRow = ureq::patch(&url)
        .header("Authorization", &format!("Bearer {token}"))
        .header("Content-Type", "application/json")
        .send_json(&Body { content })?
        .body_mut()
        .read_json()?;
    Ok(resp)
}

pub fn delete_message(
    api_url: &str,
    token: &str,
    channel_id: Uuid,
    message_id: Uuid,
) -> Result<()> {
    let url = format!("{api_url}/channels/{channel_id}/messages/{message_id}");
    ureq::delete(&url)
        .header("Authorization", &format!("Bearer {token}"))
        .call()?;
    Ok(())
}

pub fn rename_channel(
    api_url: &str,
    token: &str,
    server_id: Uuid,
    channel_id: Uuid,
    name: &str,
) -> Result<ChannelRow> {
    #[derive(Serialize)]
    struct Body<'a> {
        name: &'a str,
    }
    let url = format!("{api_url}/servers/{server_id}/channels/{channel_id}");
    let resp: ChannelRow = ureq::patch(&url)
        .header("Authorization", &format!("Bearer {token}"))
        .header("Content-Type", "application/json")
        .send_json(&Body { name })?
        .body_mut()
        .read_json()?;
    Ok(resp)
}

pub fn delete_channel_api(
    api_url: &str,
    token: &str,
    server_id: Uuid,
    channel_id: Uuid,
) -> Result<()> {
    let url = format!("{api_url}/servers/{server_id}/channels/{channel_id}");
    ureq::delete(&url)
        .header("Authorization", &format!("Bearer {token}"))
        .call()?;
    Ok(())
}

pub fn rename_server(
    api_url: &str,
    token: &str,
    server_id: Uuid,
    name: &str,
) -> Result<ServerRow> {
    #[derive(Serialize)]
    struct Body<'a> {
        name: &'a str,
    }
    let url = format!("{api_url}/servers/{server_id}");
    let resp: ServerRow = ureq::patch(&url)
        .header("Authorization", &format!("Bearer {token}"))
        .header("Content-Type", "application/json")
        .send_json(&Body { name })?
        .body_mut()
        .read_json()?;
    Ok(resp)
}

pub fn delete_server_api(api_url: &str, token: &str, server_id: Uuid) -> Result<()> {
    let url = format!("{api_url}/servers/{server_id}");
    ureq::delete(&url)
        .header("Authorization", &format!("Bearer {token}"))
        .call()?;
    Ok(())
}

pub fn pin_message(
    api_url: &str,
    token: &str,
    channel_id: Uuid,
    message_id: Uuid,
) -> Result<MessageRow> {
    let url = format!("{api_url}/channels/{channel_id}/messages/{message_id}/pin");
    let resp: MessageRow = ureq::post(&url)
        .header("Authorization", &format!("Bearer {token}"))
        .send_empty()?
        .body_mut()
        .read_json()?;
    Ok(resp)
}

// ---- Invites ----

#[derive(Debug, Deserialize)]
pub struct InviteResponse {
    pub code: String,
    pub server_id: Uuid,
}

pub fn create_invite(api_url: &str, token: &str, server_id: Uuid) -> Result<InviteResponse> {
    let url = format!("{api_url}/servers/{server_id}/invites");
    let resp: InviteResponse = ureq::post(&url)
        .header("Authorization", &format!("Bearer {token}"))
        .send_empty()?
        .body_mut()
        .read_json()?;
    Ok(resp)
}

pub fn join_invite(api_url: &str, token: &str, code: &str) -> Result<()> {
    let url = format!("{api_url}/invites/{code}/join");
    ureq::post(&url)
        .header("Authorization", &format!("Bearer {token}"))
        .send_empty()?;
    Ok(())
}

// ---- DMs ----

pub fn create_dm(api_url: &str, token: &str, recipient_id: Uuid) -> Result<ChannelRow> {
    #[derive(Serialize)]
    struct Body {
        recipient_id: Uuid,
    }
    let url = format!("{api_url}/dms");
    let resp: ChannelRow = ureq::post(&url)
        .header("Authorization", &format!("Bearer {token}"))
        .header("Content-Type", "application/json")
        .send_json(&Body { recipient_id })?
        .body_mut()
        .read_json()?;
    Ok(resp)
}

#[derive(Debug, Clone, Deserialize)]
pub struct DmChannelInfo {
    pub id: Uuid,
    pub server_id: Option<Uuid>,
    pub name: String,
    pub channel_type: String,
    pub topic: Option<String>,
    pub position: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub participants: Vec<Uuid>,
}

pub fn list_dms(api_url: &str, token: &str) -> Result<Vec<DmChannelInfo>> {
    let url = format!("{api_url}/dms");
    let resp: Vec<DmChannelInfo> = ureq::get(&url)
        .header("Authorization", &format!("Bearer {token}"))
        .call()?
        .body_mut()
        .read_json()?;
    Ok(resp)
}

// ---- Members ----

#[derive(Debug, Deserialize)]
pub struct MemberWithUser {
    pub server_id: Uuid,
    pub user_id: Uuid,
    pub nickname: Option<String>,
    pub joined_at: chrono::DateTime<chrono::Utc>,
    pub username: String,
    pub discriminator: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
}

pub fn fetch_members(api_url: &str, token: &str, server_id: Uuid) -> Result<Vec<MemberWithUser>> {
    let url = format!("{api_url}/servers/{server_id}/members");
    let resp: Vec<MemberWithUser> = ureq::get(&url)
        .header("Authorization", &format!("Bearer {token}"))
        .call()?
        .body_mut()
        .read_json()?;
    Ok(resp)
}

// ---- Profile ----

#[derive(Debug, Deserialize)]
pub struct UserProfileResponse {
    pub id: Uuid,
    pub username: String,
    pub discriminator: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub email: Option<String>,
    pub bio: Option<String>,
    pub banner_url: Option<String>,
}

pub fn get_me(api_url: &str, token: &str) -> Result<UserProfileResponse> {
    let url = format!("{api_url}/users/@me");
    let resp: UserProfileResponse = ureq::get(&url)
        .header("Authorization", &format!("Bearer {token}"))
        .call()?
        .body_mut()
        .read_json()?;
    Ok(resp)
}

pub fn update_profile(
    api_url: &str,
    token: &str,
    display_name: Option<&str>,
    bio: Option<&str>,
) -> Result<UserProfileResponse> {
    #[derive(Serialize)]
    struct Body<'a> {
        #[serde(skip_serializing_if = "Option::is_none")]
        display_name: Option<&'a str>,
        #[serde(skip_serializing_if = "Option::is_none")]
        bio: Option<&'a str>,
    }
    let url = format!("{api_url}/users/@me");
    let resp: UserProfileResponse = ureq::patch(&url)
        .header("Authorization", &format!("Bearer {token}"))
        .header("Content-Type", "application/json")
        .send_json(&Body { display_name, bio })?
        .body_mut()
        .read_json()?;
    Ok(resp)
}

// ---- Image Upload ----

#[derive(Debug, Deserialize)]
pub struct ImageUploadResponse {
    pub message_id: Uuid,
    pub attachment_id: Uuid,
    pub url: String,
}

pub fn upload_file_base64(
    api_url: &str,
    token: &str,
    channel_id: Uuid,
    filename: &str,
    content_type: &str,
    base64_data: &str,
) -> Result<ImageUploadResponse> {
    #[derive(Serialize)]
    struct Body<'a> {
        filename: &'a str,
        content_type: &'a str,
        data: &'a str,
    }
    let url = format!("{api_url}/channels/{channel_id}/files");
    let resp: ImageUploadResponse = ureq::post(&url)
        .header("Authorization", &format!("Bearer {token}"))
        .header("Content-Type", "application/json")
        .send_json(&Body { filename, content_type, data: base64_data })?
        .body_mut()
        .read_json()?;
    Ok(resp)
}

/// Fetch base64 image data for an attachment.
pub fn fetch_attachment_base64(
    api_url: &str,
    token: &str,
    attachment_id: Uuid,
) -> Result<(String, String)> {
    // Returns (content_type, base64_data)
    #[derive(Deserialize)]
    struct Resp {
        content_type: String,
        data: Option<String>,
    }
    let url = format!("{api_url}/attachments/{attachment_id}/base64");
    let resp: Resp = ureq::get(&url)
        .header("Authorization", &format!("Bearer {token}"))
        .call()?
        .body_mut()
        .read_json()?;
    Ok((resp.content_type, resp.data.unwrap_or_default()))
}
