#!/usr/bin/env bash
# End-to-end test for rusteze
# Tests: auth, servers, channels, messages, edit/delete, reactions,
#        DMs, user profiles, roles, members, invites, pins, search
# Usage: ./scripts/e2e-test.sh [API_URL] [WS_URL]
set -euo pipefail

API=${1:-http://127.0.0.1:14702}
GW=${2:-ws://127.0.0.1:14703}
OK=0
FAIL=0

pass() { echo "  ✓ $1"; ((OK++)); }
fail() { echo "  ✗ $1"; ((FAIL++)); }
jq_val() { python3 -c "import sys,json; print(json.load(sys.stdin)$1)" 2>/dev/null; }

echo "=== Rusteze E2E Test ==="
echo "API: $API"
echo ""

# ---------- AUTH ----------
echo "--- Auth ---"

RESP=$(curl -s -X POST "$API/auth/register" \
  -H "Content-Type: application/json" \
  -d '{"username":"alice","email":"alice@e2e.test","password":"password123"}')
TOKEN_A=$(echo "$RESP" | jq_val "['token']")
USER_A=$(echo "$RESP" | jq_val "['user_id']")
[ -n "$TOKEN_A" ] && pass "register user A ($USER_A)" || fail "register user A"

RESP=$(curl -s -X POST "$API/auth/register" \
  -H "Content-Type: application/json" \
  -d '{"username":"bob","email":"bob@e2e.test","password":"password456"}')
TOKEN_B=$(echo "$RESP" | jq_val "['token']")
USER_B=$(echo "$RESP" | jq_val "['user_id']")
[ -n "$TOKEN_B" ] && pass "register user B ($USER_B)" || fail "register user B"

RESP=$(curl -s -X POST "$API/auth/login" \
  -H "Content-Type: application/json" \
  -d '{"email":"alice@e2e.test","password":"password123"}')
LOGIN_TOKEN=$(echo "$RESP" | jq_val "['token']")
[ -n "$LOGIN_TOKEN" ] && pass "login user A" || fail "login user A"

# ---------- USER PROFILES ----------
echo ""
echo "--- User Profiles ---"

RESP=$(curl -s "$API/users/@me" -H "Authorization: Bearer $TOKEN_A")
ME_USERNAME=$(echo "$RESP" | jq_val "['username']")
[ "$ME_USERNAME" = "alice" ] && pass "get @me" || fail "get @me (got: $ME_USERNAME)"

RESP=$(curl -s -X PATCH "$API/users/@me" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN_A" \
  -d '{"display_name":"Alice W","bio":"hello world"}')
DISPLAY=$(echo "$RESP" | jq_val "['display_name']")
[ "$DISPLAY" = "Alice W" ] && pass "update profile" || fail "update profile"

RESP=$(curl -s "$API/users/$USER_A" -H "Authorization: Bearer $TOKEN_B")
PUB_BIO=$(echo "$RESP" | jq_val "['bio']")
[ "$PUB_BIO" = "hello world" ] && pass "view other user profile" || fail "view other user profile"

# ---------- SERVERS ----------
echo ""
echo "--- Servers ---"

RESP=$(curl -s -X POST "$API/servers" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN_A" \
  -d '{"name":"Test Server"}')
SERVER_ID=$(echo "$RESP" | jq_val "['id']")
[ -n "$SERVER_ID" ] && pass "create server ($SERVER_ID)" || fail "create server"

RESP=$(curl -s "$API/servers" -H "Authorization: Bearer $TOKEN_A")
COUNT=$(echo "$RESP" | python3 -c "import sys,json; print(len(json.load(sys.stdin)))")
[ "$COUNT" -ge 1 ] && pass "list servers ($COUNT)" || fail "list servers"

# ---------- CHANNELS ----------
echo ""
echo "--- Channels ---"

RESP=$(curl -s "$API/servers/$SERVER_ID/channels" -H "Authorization: Bearer $TOKEN_A")
CHANNEL_ID=$(echo "$RESP" | jq_val "[0]['id']")
[ -n "$CHANNEL_ID" ] && pass "list channels (auto #general: $CHANNEL_ID)" || fail "list channels"

RESP=$(curl -s -X POST "$API/servers/$SERVER_ID/channels" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN_A" \
  -d '{"name":"dev","channel_type":"text"}')
CH2=$(echo "$RESP" | jq_val "['id']")
[ -n "$CH2" ] && pass "create channel #dev" || fail "create channel"

# ---------- INVITES ----------
echo ""
echo "--- Invites ---"

RESP=$(curl -s -X POST "$API/servers/$SERVER_ID/invites" \
  -H "Authorization: Bearer $TOKEN_A")
INVITE_CODE=$(echo "$RESP" | jq_val "['code']")
[ -n "$INVITE_CODE" ] && pass "create invite ($INVITE_CODE)" || fail "create invite"

RESP=$(curl -s -X POST "$API/invites/$INVITE_CODE/join" \
  -H "Authorization: Bearer $TOKEN_B")
JOINED=$(echo "$RESP" | jq_val "['server_id']")
[ "$JOINED" = "$SERVER_ID" ] && pass "user B joins via invite" || fail "user B joins"

# ---------- MEMBERS ----------
echo ""
echo "--- Members ---"

RESP=$(curl -s "$API/servers/$SERVER_ID/members" -H "Authorization: Bearer $TOKEN_A")
MCOUNT=$(echo "$RESP" | python3 -c "import sys,json; print(len(json.load(sys.stdin)))")
[ "$MCOUNT" = "2" ] && pass "list members ($MCOUNT)" || fail "list members (got $MCOUNT)"

# ---------- ROLES ----------
echo ""
echo "--- Roles ---"

RESP=$(curl -s -X POST "$API/servers/$SERVER_ID/roles" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN_A" \
  -d '{"name":"Moderator","color":3447003,"permissions":8}')
ROLE_ID=$(echo "$RESP" | jq_val "['id']")
[ -n "$ROLE_ID" ] && pass "create role ($ROLE_ID)" || fail "create role"

RESP=$(curl -s "$API/servers/$SERVER_ID/roles" -H "Authorization: Bearer $TOKEN_A")
RCOUNT=$(echo "$RESP" | python3 -c "import sys,json; print(len(json.load(sys.stdin)))")
[ "$RCOUNT" -ge 1 ] && pass "list roles ($RCOUNT)" || fail "list roles"

RESP=$(curl -s -X PUT "$API/servers/$SERVER_ID/roles/$ROLE_ID/members" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN_A" \
  -d "{\"user_id\":\"$USER_B\"}")
ASSIGN_OK=$(echo "$RESP" | jq_val "['ok']")
[ "$ASSIGN_OK" = "True" ] && pass "assign role to user B" || fail "assign role"

# ---------- MESSAGES ----------
echo ""
echo "--- Messages ---"

RESP=$(curl -s -X POST "$API/channels/$CHANNEL_ID/messages" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN_A" \
  -d '{"content":"Hello from Alice!"}')
MSG_ID=$(echo "$RESP" | jq_val "['id']")
[ -n "$MSG_ID" ] && pass "send message ($MSG_ID)" || fail "send message"

RESP=$(curl -s "$API/channels/$CHANNEL_ID/messages" \
  -H "Authorization: Bearer $TOKEN_B")
MSGCOUNT=$(echo "$RESP" | python3 -c "import sys,json; print(len(json.load(sys.stdin)))")
[ "$MSGCOUNT" -ge 1 ] && pass "list messages ($MSGCOUNT)" || fail "list messages"

# Edit message
RESP=$(curl -s -X PATCH "$API/channels/$CHANNEL_ID/messages/$MSG_ID" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN_A" \
  -d '{"content":"Hello from Alice! (edited)"}')
EDITED=$(echo "$RESP" | jq_val "['content']")
[ "$EDITED" = "Hello from Alice! (edited)" ] && pass "edit message" || fail "edit message"

# Pin message
RESP=$(curl -s -X POST "$API/channels/$CHANNEL_ID/messages/$MSG_ID/pin" \
  -H "Authorization: Bearer $TOKEN_A")
PINNED=$(echo "$RESP" | jq_val "['pinned']")
[ "$PINNED" = "True" ] && pass "pin message" || fail "pin message"

# List pins
RESP=$(curl -s "$API/channels/$CHANNEL_ID/pins" -H "Authorization: Bearer $TOKEN_A")
PCOUNT=$(echo "$RESP" | python3 -c "import sys,json; print(len(json.load(sys.stdin)))")
[ "$PCOUNT" -ge 1 ] && pass "list pins ($PCOUNT)" || fail "list pins"

# Search messages
RESP=$(curl -s "$API/channels/$CHANNEL_ID/search?q=Alice" \
  -H "Authorization: Bearer $TOKEN_A")
SCOUNT=$(echo "$RESP" | python3 -c "import sys,json; print(len(json.load(sys.stdin)))")
[ "$SCOUNT" -ge 1 ] && pass "search messages ($SCOUNT results)" || fail "search messages"

# ---------- REACTIONS ----------
echo ""
echo "--- Reactions ---"

RESP=$(curl -s -X PUT "$API/channels/$CHANNEL_ID/messages/$MSG_ID/reactions" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN_A" \
  -d '{"emoji":"👍"}')
R_OK=$(echo "$RESP" | jq_val "['ok']")
[ "$R_OK" = "True" ] && pass "add reaction" || fail "add reaction"

RESP=$(curl -s "$API/channels/$CHANNEL_ID/messages/$MSG_ID/reactions" \
  -H "Authorization: Bearer $TOKEN_A")
R_COUNT=$(echo "$RESP" | python3 -c "import sys,json; print(json.load(sys.stdin)[0]['count'])")
[ "$R_COUNT" = "1" ] && pass "list reactions (count=$R_COUNT)" || fail "list reactions"

# ---------- DMs ----------
echo ""
echo "--- Direct Messages ---"

RESP=$(curl -s -X POST "$API/dms" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN_A" \
  -d "{\"recipient_id\":\"$USER_B\"}")
DM_ID=$(echo "$RESP" | jq_val "['id']")
[ -n "$DM_ID" ] && pass "create DM ($DM_ID)" || fail "create DM"

RESP=$(curl -s -X POST "$API/channels/$DM_ID/messages" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN_A" \
  -d '{"content":"Hey Bob, private message!"}')
DM_MSG=$(echo "$RESP" | jq_val "['id']")
[ -n "$DM_MSG" ] && pass "send DM message" || fail "send DM message"

RESP=$(curl -s "$API/dms" -H "Authorization: Bearer $TOKEN_A")
DM_COUNT=$(echo "$RESP" | python3 -c "import sys,json; print(len(json.load(sys.stdin)))")
[ "$DM_COUNT" -ge 1 ] && pass "list DMs ($DM_COUNT)" || fail "list DMs"

# ---------- DELETE MESSAGE ----------
echo ""
echo "--- Delete ---"

RESP=$(curl -s -X POST "$API/channels/$CHANNEL_ID/messages" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN_A" \
  -d '{"content":"temporary message"}')
TEMP_MSG=$(echo "$RESP" | jq_val "['id']")

RESP=$(curl -s -X DELETE "$API/channels/$CHANNEL_ID/messages/$TEMP_MSG" \
  -H "Authorization: Bearer $TOKEN_A")
DEL=$(echo "$RESP" | jq_val "['deleted']")
[ "$DEL" = "True" ] && pass "delete message" || fail "delete message"

# ---------- WEBSOCKET ----------
echo ""
echo "--- WebSocket ---"

if command -v websocat &>/dev/null; then
  READY=$(echo "{\"type\":\"Authenticate\",\"token\":\"$TOKEN_B\"}" | \
    timeout 3 websocat -1 "$GW" 2>/dev/null || true)
  if echo "$READY" | python3 -c "import sys,json; d=json.load(sys.stdin); assert d['type']=='Ready'; assert d['user']['username']=='bob'" 2>/dev/null; then
    pass "gateway Ready (real user data)"
  else
    fail "gateway Ready"
  fi
else
  echo "  - websocat not installed, skipping WS test"
fi

# ---------- SUMMARY ----------
echo ""
echo "================================="
echo "  Passed: $OK"
echo "  Failed: $FAIL"
echo "================================="
[ $FAIL -eq 0 ] && echo "All tests passed!" || exit 1
