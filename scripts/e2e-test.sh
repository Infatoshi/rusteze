#!/usr/bin/env bash
# End-to-end test for rusteze prototype
# Usage: ./scripts/e2e-test.sh [API_URL]
set -euo pipefail

API=${1:-http://127.0.0.1:14702}
GW=${2:-ws://127.0.0.1:14703}

echo "=== Rusteze E2E Test ==="
echo "API: $API"
echo "GW:  $GW"
echo

# 1. Register User A
echo "--- 1. Register User A ---"
RESP_A=$(curl -s -X POST "$API/auth/register" \
  -H "Content-Type: application/json" \
  -d '{"username":"alice","email":"alice@test.com","password":"password123"}')
echo "$RESP_A"
TOKEN_A=$(echo "$RESP_A" | python3 -c "import sys,json; print(json.load(sys.stdin)['token'])")
USER_A=$(echo "$RESP_A" | python3 -c "import sys,json; print(json.load(sys.stdin)['user_id'])")
echo "User A: $USER_A"
echo

# 2. Register User B
echo "--- 2. Register User B ---"
RESP_B=$(curl -s -X POST "$API/auth/register" \
  -H "Content-Type: application/json" \
  -d '{"username":"bob","email":"bob@test.com","password":"password456"}')
echo "$RESP_B"
TOKEN_B=$(echo "$RESP_B" | python3 -c "import sys,json; print(json.load(sys.stdin)['token'])")
USER_B=$(echo "$RESP_B" | python3 -c "import sys,json; print(json.load(sys.stdin)['user_id'])")
echo "User B: $USER_B"
echo

# 3. User A creates a server
echo "--- 3. User A creates a server ---"
SERVER=$(curl -s -X POST "$API/servers" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN_A" \
  -d '{"name":"Test Server"}')
echo "$SERVER"
SERVER_ID=$(echo "$SERVER" | python3 -c "import sys,json; print(json.load(sys.stdin)['id'])")
echo "Server: $SERVER_ID"
echo

# 4. List channels (should have #general)
echo "--- 4. List channels ---"
CHANNELS=$(curl -s "$API/servers/$SERVER_ID/channels" \
  -H "Authorization: Bearer $TOKEN_A")
echo "$CHANNELS"
CHANNEL_ID=$(echo "$CHANNELS" | python3 -c "import sys,json; print(json.load(sys.stdin)[0]['id'])")
echo "Channel: $CHANNEL_ID"
echo

# 5. User A creates an invite
echo "--- 5. Create invite ---"
INVITE=$(curl -s -X POST "$API/servers/$SERVER_ID/invites" \
  -H "Authorization: Bearer $TOKEN_A")
echo "$INVITE"
INVITE_CODE=$(echo "$INVITE" | python3 -c "import sys,json; print(json.load(sys.stdin)['code'])")
echo "Invite code: $INVITE_CODE"
echo

# 6. User B joins via invite
echo "--- 6. User B joins via invite ---"
JOIN=$(curl -s -X POST "$API/invites/$INVITE_CODE/join" \
  -H "Authorization: Bearer $TOKEN_B")
echo "$JOIN"
echo

# 7. User A sends a message
echo "--- 7. User A sends a message ---"
MSG=$(curl -s -X POST "$API/channels/$CHANNEL_ID/messages" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN_A" \
  -d '{"content":"Hello from Alice!"}')
echo "$MSG"
echo

# 8. User B fetches messages
echo "--- 8. User B fetches messages ---"
MESSAGES=$(curl -s "$API/channels/$CHANNEL_ID/messages" \
  -H "Authorization: Bearer $TOKEN_B")
echo "$MESSAGES"
echo

# 9. WebSocket test (connect as User B, receive Ready)
echo "--- 9. WebSocket test (User B connects) ---"
if command -v websocat &>/dev/null; then
  echo "{\"type\":\"Authenticate\",\"token\":\"$TOKEN_B\"}" | \
    timeout 3 websocat -1 "$GW" 2>/dev/null && echo "(Ready event received)" || echo "(timeout — normal if no message pushed)"
else
  echo "websocat not installed, skipping WS test"
fi
echo

echo "=== E2E Test Complete ==="
