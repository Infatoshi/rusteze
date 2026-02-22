//! Integration tests for the rusteze client.
//!
//! These require the backend to be running on localhost:
//!   cargo run -p rusteze-server &
//!   cargo run -p rusteze-gateway &
//!
//! Run with: cargo test -p rusteze-client -- --ignored

// The api module is part of the binary crate, so we use ureq directly
// with the same patterns. We re-implement the helpers here to avoid
// needing to restructure the crate just for tests.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

const API: &str = "http://127.0.0.1:14702";
const GW: &str = "ws://127.0.0.1:14703";

// ---- Helpers ----

#[derive(Debug, Deserialize)]
struct AuthResponse {
    user_id: Uuid,
    token: String,
}

#[derive(Debug, Deserialize)]
struct ServerRow {
    id: Uuid,
    name: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ChannelRow {
    id: Uuid,
    name: String,
    channel_type: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct MessageRow {
    id: Uuid,
    channel_id: Uuid,
    content: Option<String>,
    edited_at: Option<String>,
}

fn unique_email() -> String {
    format!("test-{}@e2e.local", Uuid::new_v4())
}

fn register(email: &str) -> AuthResponse {
    #[derive(Serialize)]
    struct Body<'a> {
        username: &'a str,
        email: &'a str,
        password: &'a str,
    }
    let username = email.split('@').next().unwrap();
    ureq::post(&format!("{API}/auth/register"))
        .header("Content-Type", "application/json")
        .send_json(&Body {
            username,
            email,
            password: "testpass123",
        })
        .expect("register failed")
        .body_mut()
        .read_json::<AuthResponse>()
        .expect("parse register response")
}

fn create_server(token: &str, name: &str) -> ServerRow {
    #[derive(Serialize)]
    struct Body<'a> {
        name: &'a str,
    }
    ureq::post(&format!("{API}/servers"))
        .header("Authorization", &format!("Bearer {token}"))
        .header("Content-Type", "application/json")
        .send_json(&Body { name })
        .expect("create server failed")
        .body_mut()
        .read_json::<ServerRow>()
        .expect("parse server response")
}

fn fetch_servers(token: &str) -> Vec<ServerRow> {
    ureq::get(&format!("{API}/servers"))
        .header("Authorization", &format!("Bearer {token}"))
        .call()
        .expect("fetch servers failed")
        .body_mut()
        .read_json::<Vec<ServerRow>>()
        .expect("parse servers")
}

fn fetch_channels(token: &str, server_id: Uuid) -> Vec<ChannelRow> {
    ureq::get(&format!("{API}/servers/{server_id}/channels"))
        .header("Authorization", &format!("Bearer {token}"))
        .call()
        .expect("fetch channels failed")
        .body_mut()
        .read_json::<Vec<ChannelRow>>()
        .expect("parse channels")
}

fn send_message(token: &str, channel_id: Uuid, content: &str) -> MessageRow {
    #[derive(Serialize)]
    struct Body<'a> {
        content: &'a str,
    }
    ureq::post(&format!("{API}/channels/{channel_id}/messages"))
        .header("Authorization", &format!("Bearer {token}"))
        .header("Content-Type", "application/json")
        .send_json(&Body { content })
        .expect("send message failed")
        .body_mut()
        .read_json::<MessageRow>()
        .expect("parse message")
}

fn fetch_messages(token: &str, channel_id: Uuid) -> Vec<MessageRow> {
    ureq::get(&format!("{API}/channels/{channel_id}/messages?limit=50"))
        .header("Authorization", &format!("Bearer {token}"))
        .call()
        .expect("fetch messages failed")
        .body_mut()
        .read_json::<Vec<MessageRow>>()
        .expect("parse messages")
}

fn edit_message(token: &str, channel_id: Uuid, message_id: Uuid, content: &str) -> MessageRow {
    #[derive(Serialize)]
    struct Body<'a> {
        content: &'a str,
    }
    ureq::patch(&format!("{API}/channels/{channel_id}/messages/{message_id}"))
        .header("Authorization", &format!("Bearer {token}"))
        .header("Content-Type", "application/json")
        .send_json(&Body { content })
        .expect("edit message failed")
        .body_mut()
        .read_json::<MessageRow>()
        .expect("parse edited message")
}

fn delete_message(token: &str, channel_id: Uuid, message_id: Uuid) {
    ureq::delete(&format!("{API}/channels/{channel_id}/messages/{message_id}"))
        .header("Authorization", &format!("Bearer {token}"))
        .call()
        .expect("delete message failed");
}

// ---- Tests ----

#[test]
#[ignore]
fn test_register_and_login() {
    let email = unique_email();
    let reg = register(&email);
    assert!(!reg.token.is_empty(), "register should return a token");

    // Login with same credentials
    #[derive(Serialize)]
    struct LoginBody<'a> {
        email: &'a str,
        password: &'a str,
    }
    let login: AuthResponse = ureq::post(&format!("{API}/auth/login"))
        .header("Content-Type", "application/json")
        .send_json(&LoginBody {
            email: &email,
            password: "testpass123",
        })
        .expect("login failed")
        .body_mut()
        .read_json()
        .expect("parse login");

    assert_eq!(login.user_id, reg.user_id, "login should return same user_id");
    assert!(!login.token.is_empty(), "login should return a token");
}

#[test]
#[ignore]
fn test_create_server_and_channels() {
    let email = unique_email();
    let auth = register(&email);

    let server = create_server(&auth.token, "Integration Test Server");
    assert_eq!(server.name, "Integration Test Server");

    // Verify it shows up in server list
    let servers = fetch_servers(&auth.token);
    assert!(
        servers.iter().any(|s| s.id == server.id),
        "created server should appear in list"
    );

    // Verify auto-created #general channel
    let channels = fetch_channels(&auth.token, server.id);
    assert!(
        channels.iter().any(|c| c.name == "general" && c.channel_type == "text"),
        "server should have auto-created #general text channel"
    );
}

#[test]
#[ignore]
fn test_send_and_fetch_messages() {
    let email = unique_email();
    let auth = register(&email);
    let server = create_server(&auth.token, "Msg Test Server");
    let channels = fetch_channels(&auth.token, server.id);
    let general = channels
        .iter()
        .find(|c| c.name == "general")
        .expect("should have #general");

    // Send a message
    let msg = send_message(&auth.token, general.id, "Hello from integration test!");
    assert_eq!(msg.content.as_deref(), Some("Hello from integration test!"));

    // Fetch it back
    let messages = fetch_messages(&auth.token, general.id);
    assert!(
        messages.iter().any(|m| m.id == msg.id),
        "sent message should appear in fetch"
    );
    let fetched = messages.iter().find(|m| m.id == msg.id).unwrap();
    assert_eq!(
        fetched.content.as_deref(),
        Some("Hello from integration test!")
    );
}

#[test]
#[ignore]
fn test_edit_and_delete_message() {
    let email = unique_email();
    let auth = register(&email);
    let server = create_server(&auth.token, "Edit Test Server");
    let channels = fetch_channels(&auth.token, server.id);
    let general = channels.iter().find(|c| c.name == "general").unwrap();

    // Send
    let msg = send_message(&auth.token, general.id, "original content");
    assert_eq!(msg.content.as_deref(), Some("original content"));

    // Edit
    let edited = edit_message(&auth.token, general.id, msg.id, "edited content");
    assert_eq!(edited.content.as_deref(), Some("edited content"));
    assert!(edited.edited_at.is_some(), "edited_at should be set");

    // Delete
    delete_message(&auth.token, general.id, msg.id);

    // Verify it's gone
    let messages = fetch_messages(&auth.token, general.id);
    assert!(
        !messages.iter().any(|m| m.id == msg.id),
        "deleted message should not appear"
    );
}

#[test]
#[ignore]
fn test_gateway_receives_message() {
    use rusteze_models::{ClientEvent, ServerEvent};
    use tungstenite::connect;

    let email = unique_email();
    let auth = register(&email);
    let server = create_server(&auth.token, "GW Test Server");
    let channels = fetch_channels(&auth.token, server.id);
    let general = channels.iter().find(|c| c.name == "general").unwrap();

    // Connect WebSocket
    let (mut ws, _) = connect(GW).expect("ws connect failed");

    // Authenticate
    let auth_event = ClientEvent::Authenticate {
        token: auth.token.clone(),
    };
    let auth_json = serde_json::to_string(&auth_event).unwrap();
    ws.send(tungstenite::Message::Text(auth_json.into())).unwrap();

    // Read Ready event
    let ready_msg = ws.read().expect("should receive Ready");
    let ready_text = ready_msg.into_text().expect("should be text");
    let ready: ServerEvent = serde_json::from_str(&ready_text).expect("parse Ready");
    match &ready {
        ServerEvent::Ready { servers, .. } => {
            assert!(
                servers.iter().any(|s| s.id == server.id),
                "Ready should contain our server"
            );
        }
        other => panic!("expected Ready, got {:?}", std::mem::discriminant(other)),
    }

    // Send a message via HTTP (this should trigger a MessageCreate on the WS)
    let sent = send_message(&auth.token, general.id, "gateway test message");

    // Read the MessageCreate event from the gateway
    // Set a read timeout on the underlying TCP stream
    match ws.get_mut() {
        tungstenite::stream::MaybeTlsStream::Plain(tcp) => {
            tcp.set_read_timeout(Some(std::time::Duration::from_secs(5))).ok();
        }
        _ => {}
    }

    let event_msg = ws.read().expect("should receive MessageCreate");
    let event_text = event_msg.into_text().expect("should be text");
    let event: ServerEvent = serde_json::from_str(&event_text).expect("parse event");
    match event {
        ServerEvent::MessageCreate(msg) => {
            assert_eq!(msg.id, sent.id, "should receive the message we sent");
            assert_eq!(
                msg.content.as_deref(),
                Some("gateway test message")
            );
        }
        other => panic!(
            "expected MessageCreate, got {:?}",
            std::mem::discriminant(&other)
        ),
    }

    let _ = ws.close(None);
}

// ========== Channel CRUD ==========

#[test]
#[ignore]
fn test_create_text_channel() {
    let auth = register(&unique_email());
    let server = create_server(&auth.token, "ChCRUD Text");
    
    #[derive(Serialize)]
    struct Body<'a> { name: &'a str, channel_type: &'a str }
    let ch: ChannelRow = ureq::post(&format!("{API}/servers/{}/channels", server.id))
        .header("Authorization", &format!("Bearer {}", auth.token))
        .header("Content-Type", "application/json")
        .send_json(&Body { name: "dev", channel_type: "text" })
        .expect("create channel").body_mut().read_json().unwrap();
    
    assert_eq!(ch.name, "dev");
    assert_eq!(ch.channel_type, "text");
    
    let channels = fetch_channels(&auth.token, server.id);
    assert!(channels.iter().any(|c| c.name == "dev"), "new channel should appear in list");
}

#[test]
#[ignore]
fn test_create_voice_channel() {
    let auth = register(&unique_email());
    let server = create_server(&auth.token, "ChCRUD Voice");
    
    #[derive(Serialize)]
    struct Body<'a> { name: &'a str, channel_type: &'a str }
    let ch: ChannelRow = ureq::post(&format!("{API}/servers/{}/channels", server.id))
        .header("Authorization", &format!("Bearer {}", auth.token))
        .header("Content-Type", "application/json")
        .send_json(&Body { name: "gaming", channel_type: "voice" })
        .expect("create voice channel").body_mut().read_json().unwrap();
    
    assert_eq!(ch.name, "gaming");
    assert_eq!(ch.channel_type, "voice");
}

#[test]
#[ignore]
fn test_rename_channel() {
    let auth = register(&unique_email());
    let server = create_server(&auth.token, "ChCRUD Rename");
    let channels = fetch_channels(&auth.token, server.id);
    let general = channels.iter().find(|c| c.name == "general").unwrap();
    
    #[derive(Serialize)]
    struct Body<'a> { name: &'a str }
    let updated: ChannelRow = ureq::patch(&format!("{API}/servers/{}/channels/{}", server.id, general.id))
        .header("Authorization", &format!("Bearer {}", auth.token))
        .header("Content-Type", "application/json")
        .send_json(&Body { name: "announcements" })
        .expect("rename channel").body_mut().read_json().unwrap();
    
    assert_eq!(updated.name, "announcements");
    
    let channels = fetch_channels(&auth.token, server.id);
    assert!(channels.iter().any(|c| c.name == "announcements"), "renamed channel should appear");
    assert!(!channels.iter().any(|c| c.name == "general"), "old name should be gone");
}

#[test]
#[ignore]
fn test_delete_channel() {
    let auth = register(&unique_email());
    let server = create_server(&auth.token, "ChCRUD Delete");
    
    // Create a channel to delete
    #[derive(Serialize)]
    struct CBody<'a> { name: &'a str, channel_type: &'a str }
    let ch: ChannelRow = ureq::post(&format!("{API}/servers/{}/channels", server.id))
        .header("Authorization", &format!("Bearer {}", auth.token))
        .header("Content-Type", "application/json")
        .send_json(&CBody { name: "temp", channel_type: "text" })
        .expect("create").body_mut().read_json().unwrap();
    
    // Delete it
    ureq::delete(&format!("{API}/servers/{}/channels/{}", server.id, ch.id))
        .header("Authorization", &format!("Bearer {}", auth.token))
        .call()
        .expect("delete channel failed");
    
    let channels = fetch_channels(&auth.token, server.id);
    assert!(!channels.iter().any(|c| c.id == ch.id), "deleted channel should be gone");
}

// ========== Server CRUD ==========

#[test]
#[ignore]
fn test_rename_server() {
    let auth = register(&unique_email());
    let server = create_server(&auth.token, "OldName");
    
    #[derive(Serialize)]
    struct Body<'a> { name: &'a str }
    #[derive(Deserialize)]
    struct SrvResp { name: String }
    
    let updated: SrvResp = ureq::patch(&format!("{API}/servers/{}", server.id))
        .header("Authorization", &format!("Bearer {}", auth.token))
        .header("Content-Type", "application/json")
        .send_json(&Body { name: "NewName" })
        .expect("rename server").body_mut().read_json().unwrap();
    
    assert_eq!(updated.name, "NewName");
}

#[test]
#[ignore]
fn test_delete_server() {
    let auth = register(&unique_email());
    let server = create_server(&auth.token, "ToDelete");
    
    ureq::delete(&format!("{API}/servers/{}", server.id))
        .header("Authorization", &format!("Bearer {}", auth.token))
        .call()
        .expect("delete server");
    
    let servers = fetch_servers(&auth.token);
    assert!(!servers.iter().any(|s| s.id == server.id), "deleted server should be gone");
}

// ========== Invites ==========

#[test]
#[ignore]
fn test_invite_flow() {
    let auth_a = register(&unique_email());
    let auth_b = register(&unique_email());
    let server = create_server(&auth_a.token, "InviteTest");
    
    // Create invite
    #[derive(Deserialize)]
    struct InvResp { code: String }
    let invite: InvResp = ureq::post(&format!("{API}/servers/{}/invites", server.id))
        .header("Authorization", &format!("Bearer {}", auth_a.token))
        .send_empty()
        .expect("create invite").body_mut().read_json().unwrap();
    
    assert!(!invite.code.is_empty());
    
    // User B joins
    ureq::post(&format!("{API}/invites/{}/join", invite.code))
        .header("Authorization", &format!("Bearer {}", auth_b.token))
        .send_empty()
        .expect("join invite");
    
    // Verify B is now a member
    let b_servers = fetch_servers(&auth_b.token);
    assert!(b_servers.iter().any(|s| s.id == server.id), "B should see the server");
}

// ========== Members ==========

#[test]
#[ignore]
fn test_leave_server() {
    let auth_a = register(&unique_email());
    let auth_b = register(&unique_email());
    let server = create_server(&auth_a.token, "LeaveTest");
    
    // Create invite and have B join
    #[derive(Deserialize)]
    struct InvResp { code: String }
    let invite: InvResp = ureq::post(&format!("{API}/servers/{}/invites", server.id))
        .header("Authorization", &format!("Bearer {}", auth_a.token))
        .send_empty().unwrap().body_mut().read_json().unwrap();
    ureq::post(&format!("{API}/invites/{}/join", invite.code))
        .header("Authorization", &format!("Bearer {}", auth_b.token))
        .send_empty().unwrap();
    
    // B leaves
    ureq::delete(&format!("{API}/servers/{}/members/@me", server.id))
        .header("Authorization", &format!("Bearer {}", auth_b.token))
        .call()
        .expect("leave server");
    
    let b_servers = fetch_servers(&auth_b.token);
    assert!(!b_servers.iter().any(|s| s.id == server.id), "B should no longer see server");
}

// ========== Reactions ==========

#[test]
#[ignore]
fn test_add_and_remove_reaction() {
    let auth = register(&unique_email());
    let server = create_server(&auth.token, "ReactTest");
    let channels = fetch_channels(&auth.token, server.id);
    let general = channels.iter().find(|c| c.name == "general").unwrap();
    let msg = send_message(&auth.token, general.id, "react to this");
    
    // Add reaction
    #[derive(Serialize)]
    struct RBody<'a> { emoji: &'a str }
    ureq::put(&format!("{API}/channels/{}/messages/{}/reactions", general.id, msg.id))
        .header("Authorization", &format!("Bearer {}", auth.token))
        .header("Content-Type", "application/json")
        .send_json(&RBody { emoji: "👍" })
        .expect("add reaction");
    
    // List reactions
    #[derive(Deserialize)]
    struct RCount { emoji: String, count: i64 }
    let reactions: Vec<RCount> = ureq::get(&format!("{API}/channels/{}/messages/{}/reactions", general.id, msg.id))
        .header("Authorization", &format!("Bearer {}", auth.token))
        .call().unwrap().body_mut().read_json().unwrap();
    assert!(reactions.iter().any(|r| r.emoji == "👍" && r.count == 1));
    
    // Remove reaction
    ureq::delete(&format!("{API}/channels/{}/messages/{}/reactions/{}", general.id, msg.id, "👍"))
        .header("Authorization", &format!("Bearer {}", auth.token))
        .call()
        .expect("remove reaction");
    
    let reactions: Vec<RCount> = ureq::get(&format!("{API}/channels/{}/messages/{}/reactions", general.id, msg.id))
        .header("Authorization", &format!("Bearer {}", auth.token))
        .call().unwrap().body_mut().read_json().unwrap();
    assert!(!reactions.iter().any(|r| r.emoji == "👍"), "reaction should be gone");
}

// ========== DMs ==========

#[test]
#[ignore]
fn test_create_and_use_dm() {
    let auth_a = register(&unique_email());
    let auth_b = register(&unique_email());
    
    // A creates DM with B
    #[derive(Serialize)]
    struct DmBody { recipient_id: Uuid }
    #[derive(Deserialize)]
    struct DmResp { id: Uuid }
    let dm: DmResp = ureq::post(&format!("{API}/dms"))
        .header("Authorization", &format!("Bearer {}", auth_a.token))
        .header("Content-Type", "application/json")
        .send_json(&DmBody { recipient_id: auth_b.user_id })
        .expect("create DM").body_mut().read_json().unwrap();
    
    // A sends message in DM
    let msg = send_message(&auth_a.token, dm.id, "hello from DM");
    assert_eq!(msg.content.as_deref(), Some("hello from DM"));
    
    // B can read messages in the DM
    let messages = fetch_messages(&auth_b.token, dm.id);
    assert!(messages.iter().any(|m| m.content.as_deref() == Some("hello from DM")));
}

// ========== Profile ==========

#[test]
#[ignore]
fn test_update_profile() {
    let auth = register(&unique_email());
    
    #[derive(Serialize)]
    struct PBody<'a> { display_name: &'a str }
    #[derive(Deserialize)]
    struct PResp { display_name: Option<String> }
    
    let updated: PResp = ureq::patch(&format!("{API}/users/@me"))
        .header("Authorization", &format!("Bearer {}", auth.token))
        .header("Content-Type", "application/json")
        .send_json(&PBody { display_name: "Cool Name" })
        .expect("update profile").body_mut().read_json().unwrap();
    
    assert_eq!(updated.display_name.as_deref(), Some("Cool Name"));
}

// ========== Pins ==========

#[test]
#[ignore]
fn test_pin_and_list_pins() {
    let auth = register(&unique_email());
    let server = create_server(&auth.token, "PinTest");
    let channels = fetch_channels(&auth.token, server.id);
    let general = channels.iter().find(|c| c.name == "general").unwrap();
    let msg = send_message(&auth.token, general.id, "pin me");
    
    // Pin
    #[derive(Deserialize)]
    struct PinResp { pinned: bool }
    let pinned: PinResp = ureq::post(&format!("{API}/channels/{}/messages/{}/pin", general.id, msg.id))
        .header("Authorization", &format!("Bearer {}", auth.token))
        .send_empty()
        .expect("pin").body_mut().read_json().unwrap();
    assert!(pinned.pinned);
    
    // List pins
    let pins: Vec<MessageRow> = ureq::get(&format!("{API}/channels/{}/pins", general.id))
        .header("Authorization", &format!("Bearer {}", auth.token))
        .call().unwrap().body_mut().read_json().unwrap();
    assert!(pins.iter().any(|m| m.id == msg.id), "pinned message should be in pins");
}

// ========== Search ==========

#[test]
#[ignore]
fn test_search_messages() {
    let auth = register(&unique_email());
    let server = create_server(&auth.token, "SearchTest");
    let channels = fetch_channels(&auth.token, server.id);
    let general = channels.iter().find(|c| c.name == "general").unwrap();
    
    let unique_word = format!("xyzzy{}", Uuid::new_v4().to_string().replace('-', "").chars().take(8).collect::<String>());
    send_message(&auth.token, general.id, &format!("find the {unique_word} here"));
    
    let results: Vec<MessageRow> = ureq::get(&format!("{API}/channels/{}/search?q={unique_word}", general.id))
        .header("Authorization", &format!("Bearer {}", auth.token))
        .call().unwrap().body_mut().read_json().unwrap();
    assert!(!results.is_empty(), "search should find the message");
}

// ========== Kick Member ==========

#[test]
#[ignore]
fn test_kick_member() {
    let auth_a = register(&unique_email());
    let auth_b = register(&unique_email());
    let server = create_server(&auth_a.token, "KickTest");

    // Invite B
    #[derive(Deserialize)]
    struct InvResp { code: String }
    let invite: InvResp = ureq::post(&format!("{API}/servers/{}/invites", server.id))
        .header("Authorization", &format!("Bearer {}", auth_a.token))
        .send_empty().unwrap().body_mut().read_json().unwrap();
    ureq::post(&format!("{API}/invites/{}/join", invite.code))
        .header("Authorization", &format!("Bearer {}", auth_b.token))
        .send_empty().unwrap();

    // Verify B is a member
    let b_servers = fetch_servers(&auth_b.token);
    assert!(b_servers.iter().any(|s| s.id == server.id));

    // A kicks B
    ureq::delete(&format!("{API}/servers/{}/members/{}", server.id, auth_b.user_id))
        .header("Authorization", &format!("Bearer {}", auth_a.token))
        .call()
        .expect("kick failed");

    // Verify B is no longer a member
    let b_servers = fetch_servers(&auth_b.token);
    assert!(!b_servers.iter().any(|s| s.id == server.id), "kicked user should not see server");
}

// ========== Roles ==========

#[test]
#[ignore]
fn test_role_crud() {
    let auth = register(&unique_email());
    let server = create_server(&auth.token, "RoleTest");

    // Create role
    #[derive(Serialize)]
    struct RoleBody<'a> { name: &'a str, permissions: Option<i64> }
    #[derive(Deserialize)]
    struct RoleResp { id: Uuid, name: String }

    let role: RoleResp = ureq::post(&format!("{API}/servers/{}/roles", server.id))
        .header("Authorization", &format!("Bearer {}", auth.token))
        .header("Content-Type", "application/json")
        .send_json(&RoleBody { name: "Moderator", permissions: Some(8) })
        .expect("create role").body_mut().read_json().unwrap();
    assert_eq!(role.name, "Moderator");

    // List roles
    let roles: Vec<RoleResp> = ureq::get(&format!("{API}/servers/{}/roles", server.id))
        .header("Authorization", &format!("Bearer {}", auth.token))
        .call().unwrap().body_mut().read_json().unwrap();
    assert!(roles.iter().any(|r| r.name == "Moderator"));

    // Update role
    #[derive(Serialize)]
    struct UpdateBody<'a> { name: Option<&'a str> }
    let updated: RoleResp = ureq::patch(&format!("{API}/servers/{}/roles/{}", server.id, role.id))
        .header("Authorization", &format!("Bearer {}", auth.token))
        .header("Content-Type", "application/json")
        .send_json(&UpdateBody { name: Some("Admin") })
        .expect("update role").body_mut().read_json().unwrap();
    assert_eq!(updated.name, "Admin");

    // Delete role
    ureq::delete(&format!("{API}/servers/{}/roles/{}", server.id, role.id))
        .header("Authorization", &format!("Bearer {}", auth.token))
        .call().expect("delete role");

    let roles: Vec<RoleResp> = ureq::get(&format!("{API}/servers/{}/roles", server.id))
        .header("Authorization", &format!("Bearer {}", auth.token))
        .call().unwrap().body_mut().read_json().unwrap();
    assert!(!roles.iter().any(|r| r.name == "Admin"), "deleted role should be gone");
}

// ========== Group DM ==========

#[test]
#[ignore]
fn test_group_dm() {
    let auth_a = register(&unique_email());
    let auth_b = register(&unique_email());
    let auth_c = register(&unique_email());

    #[derive(Serialize)]
    struct GdmBody<'a> { name: &'a str, member_ids: Vec<Uuid> }
    #[derive(Deserialize)]
    struct ChResp { id: Uuid, channel_type: String }

    let gdm: ChResp = ureq::post(&format!("{API}/dms/group"))
        .header("Authorization", &format!("Bearer {}", auth_a.token))
        .header("Content-Type", "application/json")
        .send_json(&GdmBody { name: "Group Chat", member_ids: vec![auth_b.user_id, auth_c.user_id] })
        .expect("create group dm").body_mut().read_json().unwrap();

    assert_eq!(gdm.channel_type, "group_dm");

    // Send message in group DM
    let msg = send_message(&auth_a.token, gdm.id, "hello group");
    assert_eq!(msg.content.as_deref(), Some("hello group"));
}

// ========== Profile Reads ==========

#[test]
#[ignore]
fn test_get_own_profile() {
    let auth = register(&unique_email());

    #[derive(Deserialize)]
    struct Profile { username: String }
    let profile: Profile = ureq::get(&format!("{API}/users/@me"))
        .header("Authorization", &format!("Bearer {}", auth.token))
        .call().unwrap().body_mut().read_json().unwrap();

    assert!(!profile.username.is_empty());
}

#[test]
#[ignore]
fn test_view_other_profile() {
    let auth_a = register(&unique_email());
    let auth_b = register(&unique_email());

    #[derive(Deserialize)]
    struct PubProfile { id: Uuid, username: String }
    let profile: PubProfile = ureq::get(&format!("{API}/users/{}", auth_b.user_id))
        .header("Authorization", &format!("Bearer {}", auth_a.token))
        .call().unwrap().body_mut().read_json().unwrap();

    assert_eq!(profile.id, auth_b.user_id);
}

// ========== Image Upload ==========

#[test]
#[ignore]
fn test_upload_base64_image() {
    let auth = register(&unique_email());
    let server = create_server(&auth.token, "ImgTest");
    let channels = fetch_channels(&auth.token, server.id);
    let general = channels.iter().find(|c| c.name == "general").unwrap();

    // Create a tiny 1x1 red PNG as base64
    let tiny_png_b64 = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8/5+hHgAHggJ/PchI7wAAAABJRU5ErkJggg==";

    #[derive(Serialize)]
    struct ImgBody<'a> { filename: &'a str, content_type: &'a str, data: &'a str }
    #[derive(Deserialize)]
    struct ImgResp { message_id: Uuid, attachment_id: Uuid, url: String }

    let resp: ImgResp = ureq::post(&format!("{API}/channels/{}/files", general.id))
        .header("Authorization", &format!("Bearer {}", auth.token))
        .header("Content-Type", "application/json")
        .send_json(&ImgBody { filename: "test.png", content_type: "image/png", data: tiny_png_b64 })
        .expect("upload image").body_mut().read_json().unwrap();

    assert!(!resp.url.is_empty());

    // Fetch the base64 back
    #[derive(Deserialize)]
    struct AttResp { data: Option<String>, content_type: String }
    let att: AttResp = ureq::get(&format!("{API}{}", resp.url.replace("/attachments/", "/attachments/") + "/base64"))
        .header("Authorization", &format!("Bearer {}", auth.token))
        .call().unwrap().body_mut().read_json().unwrap();

    assert_eq!(att.content_type, "image/png");
    assert!(att.data.is_some(), "base64 data should be stored");
    assert_eq!(att.data.unwrap(), tiny_png_b64);

    // Fetch the raw image bytes
    let raw_resp = ureq::get(&format!("{API}{}", resp.url))
        .call().unwrap();
    assert_eq!(raw_resp.status(), 200);
}

// ========== Unpin ==========

#[test]
#[ignore]
fn test_unpin_message() {
    let auth = register(&unique_email());
    let server = create_server(&auth.token, "UnpinTest");
    let channels = fetch_channels(&auth.token, server.id);
    let general = channels.iter().find(|c| c.name == "general").unwrap();
    let msg = send_message(&auth.token, general.id, "pin then unpin");

    // Pin
    #[derive(Deserialize)]
    struct PinResp { pinned: bool }
    let pinned: PinResp = ureq::post(&format!("{API}/channels/{}/messages/{}/pin", general.id, msg.id))
        .header("Authorization", &format!("Bearer {}", auth.token))
        .send_empty().unwrap().body_mut().read_json().unwrap();
    assert!(pinned.pinned);

    // Unpin (toggle)
    let unpinned: PinResp = ureq::post(&format!("{API}/channels/{}/messages/{}/pin", general.id, msg.id))
        .header("Authorization", &format!("Bearer {}", auth.token))
        .send_empty().unwrap().body_mut().read_json().unwrap();
    assert!(!unpinned.pinned);

    // Verify not in pins list
    let pins: Vec<MessageRow> = ureq::get(&format!("{API}/channels/{}/pins", general.id))
        .header("Authorization", &format!("Bearer {}", auth.token))
        .call().unwrap().body_mut().read_json().unwrap();
    assert!(!pins.iter().any(|m| m.id == msg.id));
}
