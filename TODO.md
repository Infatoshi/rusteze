# Rusteze — Complete Discord Feature Parity Roadmap

> Generated from exhaustive Discord feature audit, Zed/GPUI architecture research, and current codebase inventory.
> Status legend: ✅ Done | 🔶 Partial | ⬜ Not Started

---

## Phase 1: Core Fixes & Essentials (Highest Priority)

### 1.1 Keyboard Shortcuts (Global)
- ✅ `Cmd+Q` — quit
- ✅ `Cmd+,` — toggle settings
- ✅ `Cmd+K` — fuzzy finder (quick switcher)
- ⬜ `Cmd+F` — in-channel search
- ⬜ `Escape` — close overlays, cancel editing
- ⬜ `Up Arrow` — edit last sent message (when input is empty)
- ⬜ `Shift+Escape` — mark server as read
- ⬜ `Alt+Up/Down` — navigate channels
- ⬜ `Alt+Shift+Up/Down` — navigate to next unread channel
- ⬜ `Cmd+E` — emoji picker
- ⬜ `Cmd+Shift+U` — attach file

### 1.2 Message Input Behavior
- ✅ Enter-to-send / Cmd+Enter-to-send toggle (settings)
- ⬜ Shift+Enter always inserts newline regardless of mode
- ⬜ Markdown preview (bold, italic, code, etc.)
- ⬜ `:emoji_name:` autocomplete in input
- ⬜ `@user` mention autocomplete in input
- ⬜ `#channel` link autocomplete in input
- ⬜ Slash command framework (`/giphy`, `/shrug`, etc.)
- ⬜ Paste image from clipboard → upload
- ⬜ Drag-and-drop file onto input → upload

### 1.3 Server Strip Layout
- ✅ Server icons at top, "+" and "⚙" grouped at bottom
- ⬜ Server folders (group servers into collapsible folders)
- ⬜ DM button at top of strip (above servers)
- ⬜ Unread dot indicator on server icons
- ⬜ Mention count badge (red) on server icons
- ⬜ Drag-and-drop server reorder
- ⬜ Right-click server → context menu (settings, invite, leave, mute)

### 1.4 Real-Time Event Handling
- ✅ Gateway bidirectional channel (Subscribe on select/create)
- ✅ MessageCreate, MessageUpdate, MessageDelete events
- ✅ ChannelCreate, ChannelDelete events
- ✅ MemberJoin, MemberLeave events
- ✅ TypingStart events (send + receive)
- ⬜ PresenceUpdate display (online/offline dots in member list)
- ⬜ ServerUpdate event handling (name/icon changes)
- ⬜ Gateway reconnection on disconnect (auto-retry with backoff)
- ⬜ Heartbeat/ping keepalive (prevent timeout)

---

## Phase 2: Media & File Uploads

### 2.1 File Upload System
- ⬜ `POST /channels/{id}/attachments` — multipart upload endpoint
- ⬜ File size validation (10MB limit for free, configurable)
- ⬜ Store files via `rusteze-media` LocalStorage (already written, unused)
- ⬜ `GET /attachments/{id}` — file download/serve endpoint
- ⬜ DB: link attachments to messages (`attachments` table already exists)
- ⬜ Client: file picker button in message input
- ⬜ Client: drag-and-drop file onto message area
- ⬜ Client: paste image from clipboard
- ⬜ Client: upload progress indicator
- ⬜ Client: inline image preview in message list
- ⬜ Client: video playback (MP4, WebM, up to 10MB)
- ⬜ Client: file download link for non-previewable files
- ⬜ Spoiler-tagged attachments (blur until clicked)
- ⬜ Multiple files per message (up to 10)

### 2.2 GIF Support
- ⬜ Tenor/Giphy API integration (search endpoint)
- ⬜ GIF picker button in message input
- ⬜ GIF search with categories and trending
- ⬜ GIF auto-play toggle in settings
- ⬜ Favorite GIFs (stored per-user locally)
- ⬜ GIF rendering in message list (animated)

### 2.3 Emoji System
- ⬜ Unicode emoji picker (categorized, searchable, recent)
- ⬜ Custom server emoji upload (PNG/GIF, stored in DB)
- ⬜ Custom emoji rendering in messages
- ⬜ Emoji in reactions (click to react with emoji picker)
- ⬜ `:emoji_name:` autocomplete while typing
- ⬜ Emoji in user status
- ⬜ Skin tone variants

### 2.4 Link Previews & Embeds
- ⬜ URL detection in messages
- ⬜ Open Graph metadata fetching (title, description, image, favicon)
- ⬜ Embed rendering in message list (card with preview)
- ⬜ YouTube/video embed (inline player or thumbnail)
- ⬜ Suppress embed toggle per message
- ⬜ Rich embed API for bots/webhooks (title, description, fields, color, footer, image)

---

## Phase 3: Voice & Video (Real Implementation)

### 3.1 Voice Channels (Real Audio)
- 🔶 Local audio loopback (cpal capture/playback) — working
- ⬜ WebRTC or LiveKit integration for actual multi-user voice
- ⬜ Opus encoding/decoding for voice compression
- ⬜ Voice server selection/region
- ⬜ Voice state synchronization (who is in which channel)
- ⬜ Server mute/deafen (admin)
- ⬜ Self-mute/self-deafen
- ⬜ Per-user volume slider
- ⬜ Push-to-talk (configurable keybind)
- ⬜ Voice activity detection (auto-detect speaking)
- ⬜ Speaking indicator (green ring on avatar)
- ⬜ Input/output device selection in settings
- ⬜ Input sensitivity slider

### 3.2 Audio Processing
- ⬜ Noise suppression (RNNoise or similar Rust crate)
- ⬜ Echo cancellation
- ⬜ Automatic gain control

### 3.3 Video Calls
- ⬜ Camera toggle in voice channels
- ⬜ Video rendering (peer video feeds in grid/focus view)
- ⬜ Picture-in-picture mode

### 3.4 Screen Sharing
- ⬜ macOS ScreenCaptureKit integration (capture screen/window)
- ⬜ Share specific application window
- ⬜ Stream quality settings
- ⬜ Audio sharing (capture app audio)

---

## Phase 4: Social & DMs

### 4.1 Direct Messages
- 🔶 Backend: create/list DMs, group DMs — routes exist
- ⬜ Client: DM list in sidebar (separate from servers)
- ⬜ Client: DM button at top of server strip
- ⬜ Client: open DM with user (from member list click)
- ⬜ Client: DM message view (same as channel, but with user header)
- ⬜ Group DM creation UI (select multiple users)
- ⬜ Group DM name/icon editing
- ⬜ Voice/video call in DMs

### 4.2 Friends System
- ⬜ DB: `friendships` table (user_a, user_b, status: pending/accepted/blocked)
- ⬜ Backend: send/accept/decline/remove friend request
- ⬜ Backend: list friends (all, online, pending, blocked)
- ⬜ Client: friends list view
- ⬜ Client: add friend by username
- ⬜ Client: friend request notifications
- ⬜ Client: friend online/offline indicators

### 4.3 User Profiles
- 🔶 Backend: get/update profile routes exist
- ⬜ Client: profile popup on user click (avatar, username, bio, roles, mutual servers)
- ⬜ Client: edit own profile (display name, bio, avatar upload)
- ⬜ Avatar upload (crop, resize, store via media crate)
- ⬜ Profile banner upload
- ⬜ Custom status (emoji + text, with duration)
- ⬜ Activity/presence display

### 4.4 Blocking
- ⬜ Block user (hide messages, prevent DMs)
- ⬜ Unblock user
- ⬜ Blocked user list in settings

---

## Phase 5: Server Management

### 5.1 Server Settings UI
- ⬜ Server settings page (overlay or panel)
- ⬜ Change server name
- ⬜ Upload server icon
- ⬜ Upload server banner
- ⬜ Set server description
- ⬜ Delete server (with confirmation)
- ⬜ Transfer ownership

### 5.2 Invite System
- 🔶 Backend: create/use invites — routes exist
- ⬜ Client: "Invite People" button in server
- ⬜ Client: generate invite link with options (expiry, max uses)
- ⬜ Client: copy invite link to clipboard
- ⬜ Client: invite management (list active invites, revoke)
- ⬜ Invite link preview (server name, icon, member count)
- ⬜ Join server via invite link (deep link handling)

### 5.3 Roles & Permissions
- 🔶 Backend: CRUD roles, assign/revoke — routes exist
- ⬜ Client: role management UI (create, edit, delete, reorder)
- ⬜ Client: role color picker
- ⬜ Client: permission checkboxes per role
- ⬜ Client: assign roles to members (member context menu or profile)
- ⬜ Role colors displayed on usernames in messages and member list
- ⬜ Role hierarchy enforcement
- ⬜ Per-channel permission overrides UI
- ⬜ `@everyone` role default permissions

### 5.4 Channel Categories
- ⬜ DB: `categories` table or `parent_id` on channels
- ⬜ Backend: create/rename/delete/reorder categories
- ⬜ Client: collapsible category headers in channel list
- ⬜ Client: drag-and-drop channels between categories
- ⬜ Category-level permission overrides

### 5.5 Moderation
- 🔶 Backend: kick member — route exists
- ⬜ Backend: ban/unban member (with message delete options)
- ⬜ Backend: timeout member (temporary mute)
- ⬜ Client: ban/kick/timeout from member context menu
- ⬜ Audit log (track all admin actions)
- ⬜ AutoMod (keyword filter, spam detection)
- ⬜ Server rules channel
- ⬜ Report message/user

---

## Phase 6: Notifications & Unread State

### 6.1 Unread Tracking
- 🔶 DB: `read_states` table exists (unused)
- ⬜ Backend: `PATCH /channels/{id}/read` — update last-read message
- ⬜ Backend: return unread state in channel list
- ⬜ Client: bold channel name for unread channels
- ⬜ Client: white dot on unread channels
- ⬜ Client: red mention badge on server icons
- ⬜ Client: "mark as read" (right-click channel or Shift+Escape for server)
- ⬜ Client: unread separator line in message list ("NEW MESSAGES")

### 6.2 Notification Settings
- ⬜ Global notification preferences (sounds, desktop, badges)
- ⬜ Per-server notification level (all / mentions / nothing)
- ⬜ Per-channel notification level (inherit / all / mentions / nothing)
- ⬜ Mute server/channel (with duration)
- ⬜ Suppress @everyone / @here
- ⬜ Desktop notifications (macOS `NSUserNotification` or `UNUserNotificationCenter`)
- ⬜ Notification sounds (play audio on mention/DM)
- ⬜ Dock badge count (macOS `NSApp.dockTile.badgeLabel`)

---

## Phase 7: Search & Navigation

### 7.1 Message Search
- 🔶 Backend: full-text search per channel exists
- ⬜ Client: search UI (Cmd+F opens search bar in channel header)
- ⬜ Search filters: `from:`, `has:`, `before:`, `after:`, `in:`
- ⬜ Search result list with message previews
- ⬜ Jump to message in context from search result
- ⬜ Global search across all channels/servers

### 7.2 Message History
- ⬜ Infinite scroll / "load more" pagination in message list
- ⬜ Jump to specific date
- ⬜ Jump to pinned messages
- ⬜ Jump to first unread message

---

## Phase 8: Advanced Messaging

### 8.1 Message Formatting
- ⬜ Markdown rendering in messages:
  - Bold, italic, underline, strikethrough
  - Inline code, code blocks (with syntax highlighting)
  - Block quotes
  - Headers (h1, h2, h3)
  - Bulleted/numbered lists
  - Spoiler tags (click to reveal)
  - Masked links `[text](url)`
- ⬜ `gpui-component`'s `TextView::markdown()` for rendering

### 8.2 Threads
- ⬜ DB: thread support (threads are channels with parent_id)
- ⬜ Backend: create thread from message
- ⬜ Backend: list threads for a channel
- ⬜ Client: "Create Thread" button on messages
- ⬜ Client: thread panel (side panel or inline)
- ⬜ Thread auto-archive after inactivity

### 8.3 Reactions (Client UI)
- 🔶 Backend: add/remove/list reactions — routes exist
- ⬜ Client: reaction display on messages (emoji + count)
- ⬜ Client: click to add reaction (emoji picker)
- ⬜ Client: click existing reaction to toggle
- ⬜ Client: reaction tooltip (who reacted)

### 8.4 Message Replies
- 🔶 Backend: `replies_to` field exists on messages
- ⬜ Client: reply button on messages
- ⬜ Client: reply preview above input when replying
- ⬜ Client: replied-to message reference in message display
- ⬜ Reply with/without ping toggle

### 8.5 Message Editing (Inline)
- 🔶 Backend: edit message route exists
- ⬜ Client: press Up Arrow to edit last message
- ⬜ Client: click "Edit" on message → inline editor
- ⬜ Client: Escape to cancel edit, Enter to save

### 8.6 Pins
- ✅ Backend: pin/unpin route, list pins route
- ✅ Client: pin button on messages
- ⬜ Client: pinned messages panel (channel header button)

---

## Phase 9: Settings & Preferences

### 9.1 User Settings Pages
- 🔶 Basic settings panel with Enter-to-send toggle
- ⬜ Account settings (email, password, 2FA)
- ⬜ Profile settings (display name, avatar, bio)
- ⬜ Appearance settings:
  - Dark/light theme toggle
  - Chat font size slider
  - Compact mode toggle (IRC-style)
  - Message grouping spacing
- ⬜ Voice & Video settings:
  - Input device selector
  - Output device selector
  - Input volume slider
  - Input sensitivity
  - Push-to-talk keybind
  - Noise suppression toggle
  - Echo cancellation toggle
- ⬜ Notification settings (per above)
- ⬜ Keybind settings (view/customize all shortcuts)
- ⬜ Language/locale
- ⬜ Privacy settings (DM privacy, friend requests, activity visibility)
- ⬜ Connected accounts (OAuth apps)
- ⬜ Active sessions (view/revoke)

### 9.2 Streamer Mode
- ⬜ Toggle that hides personal info, invite links, sounds, notifications
- ⬜ Auto-enable when streaming software detected

---

## Phase 10: Platform Integration

### 10.1 macOS Native Features
- ⬜ Native macOS menu bar (File, Edit, View, Window, Help)
- ⬜ macOS notifications (`UNUserNotificationCenter`)
- ⬜ Dock badge count
- ⬜ Touch Bar support (if applicable)
- ⬜ System appearance (respect dark/light mode)
- ⬜ Spotlight/Siri Shortcuts integration
- ⬜ Drag-and-drop from Finder
- ⬜ Open at login option

### 10.2 App Distribution
- ⬜ `.app` bundle creation (cargo-bundle or custom script)
- ⬜ DMG installer
- ⬜ Code signing (Apple Developer ID)
- ⬜ Notarization (Apple notary service)
- ⬜ Auto-update mechanism (Sparkle framework or custom)
- ⬜ App icon (custom icon asset)

### 10.3 Linux Support
- ⬜ GPUI already supports Linux
- ⬜ Test and fix Linux-specific issues
- ⬜ Package: AppImage, Flatpak, or .deb

---

## Phase 11: Bots & Integrations

### 11.1 Webhook Support
- ⬜ Backend: webhook CRUD per channel
- ⬜ Backend: `POST /webhooks/{id}/{token}` — send message as webhook
- ⬜ Webhook avatar/name override per message
- ⬜ Rich embed support via webhook payload
- ⬜ Client: webhook management UI (channel settings)

### 11.2 Bot Framework
- ⬜ Bot token authentication (separate from user tokens)
- ⬜ Bot permissions system
- ⬜ Slash command registration API
- ⬜ Message component API (buttons, select menus)
- ⬜ Bot presence/status
- ⬜ Rate limiting for bot API calls

### 11.3 OAuth2 Provider
- ⬜ "Login with Rusteze" for third-party apps
- ⬜ Authorization code flow
- ⬜ Scopes: identify, email, guilds, messages.read
- ⬜ Authorized apps management in user settings

---

## Phase 12: Performance & Scale

### 12.1 Virtualized Rendering
- ⬜ Virtual list for messages (only render visible items) — use GPUI's `uniform_list`
- ⬜ Virtual list for member list
- ⬜ Lazy-load server/channel data
- ⬜ Image lazy-loading (load when scrolled into view)

### 12.2 Caching
- ⬜ Client-side message cache (per channel, with LRU eviction)
- ⬜ Client-side member cache
- ⬜ IndexedDB-style local storage (redb or rusqlite for persistent cache)
- ⬜ Cache invalidation on gateway events

### 12.3 Connection Management
- ⬜ Gateway reconnection with exponential backoff
- ⬜ Offline mode (show cached data, queue messages for send)
- ⬜ Connection quality indicator
- ⬜ Rate limiting (client-side throttle)

### 12.4 Backend Scaling
- ⬜ Connection pooling tuning (sqlx pool size)
- ⬜ Redis pub/sub scaling (consider Redis Cluster for >10k users)
- ⬜ Media CDN (Cloudflare R2 for production file storage)
- ⬜ Database read replicas for read-heavy queries
- ⬜ Horizontal gateway scaling (multiple gateway instances)

---

## Architecture Notes (from Zed Research)

### Patterns to Adopt from Zed

1. **Effect Queue** — Zed queues effects (notifications, network calls) during model updates and flushes them after, preventing reentrancy bugs. We should adopt this for our state updates.

2. **`uniform_list` with `y_flipped`** — Zed's virtualized list that renders only visible items. This is literally a chat message list renderer. We should switch from our current `div().children()` approach to `uniform_list` for messages.

3. **Peer RPC Abstraction** — Zed's WebSocket layer wraps tungstenite with request/response correlation, keepalive, backpressure, and protobuf serialization. Our gateway could adopt a similar pattern for reliability.

4. **Entity Observation** — Zed uses `cx.observe()` extensively for reactive updates. We've adopted this already — ensure ALL views observe the state entity.

5. **Background Executor** — Zed runs all I/O on `cx.background_executor()` and bridges results back via spawned foreground tasks. We do this correctly.

6. **Action Dispatch** — For global shortcuts, use `cx.on_action(TypeId, window, listener)` with `cx.bind_keys()`. Actions dispatch through the focus tree and bubble up. Element-level `on_key_down` only works when the element has focus.

### Key Architectural Decisions

- **No Electron** — pure Rust + Metal rendering via GPUI
- **No React Native** — native on all platforms
- **PostgreSQL** — single source of truth for all persistent data
- **Redis Pub/Sub** — real-time event fan-out (no RabbitMQ)
- **JWT** — stateless authentication tokens
- **cpal** — cross-platform audio I/O
- **ureq** — blocking HTTP client (called from background executor)
- **tungstenite** — WebSocket client (sync, in background thread)

---

## Current Bug Fixes Needed

1. ⬜ Gateway reconnection on disconnect (currently dies silently)
2. ⬜ Message deduplication (gateway might deliver same message twice)
3. ⬜ Stale typing indicators (clean up after 5s, currently accumulate)
4. ⬜ OAuth users can't set password (empty password_hash in DB)
5. ⬜ Invite codes not cryptographically random (use `rand::rngs::OsRng`)
6. ⬜ Member roles not loaded in gateway Ready (hardcoded `roles: vec![]`)
7. ⬜ No input validation (empty server names, channel names)
8. ⬜ No rate limiting on any API endpoint
9. ⬜ Delete message only works for author (admins should be able to delete)
10. ⬜ Channel creation "Enter" / "Esc" buttons are click-only — need keyboard Enter/Escape handling on the input
