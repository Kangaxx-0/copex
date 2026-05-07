# Copilot Backend Integration Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Route Codex CLI's LLM API calls through GitHub Copilot's backend using an existing Copilot subscription, targeting GPT-5.x-Codex models.

**Architecture:** Two-phase approach. Phase 1 captures Copilot CLI's network traffic with mitmproxy to reverse-engineer the API protocol, auth mechanism, and wire format. Phase 2 builds a Python proxy that sits between Codex and Copilot's API, handling auth token injection and format translation (if needed). Phase 3 (Rust integration) is deferred until Phases 1-2 validate feasibility.

**Tech Stack:** mitmproxy (recon), Python 3.11+ with httpx/aiohttp (proxy), Codex CLI Rust codebase (future)

---

## Context for the Implementer

### What is this project?

Codex CLI (this repo) is OpenAI's open-source CLI for AI-assisted coding. It's written in Rust and talks to OpenAI's Responses API (`/v1/responses`). It supports custom model providers via `~/.codex/config.toml`.

GitHub Copilot CLI is a separate closed-source tool that provides access to the same GPT-5.x-Codex models (and others) through a GitHub subscription. The Copilot CLI UX is inferior, but the subscription provides unlimited usage.

We want Codex's UX + Copilot's backend/subscription = best of both worlds.

### Key hypothesis

Copilot's backend likely uses Azure OpenAI, which natively supports the Responses API. If true, the proxy is trivial (just auth + URL rewriting). If Copilot only speaks Chat Completions, we need a translation layer (~300-500 lines).

### Key file locations

| Component | Path |
|-----------|------|
| Codex provider config | `codex-rs/core/src/model_provider_info.rs` |
| Codex auth system | `codex-rs/login/src/auth/manager.rs` |
| Codex API client | `codex-rs/codex-api/src/endpoint/responses.rs` |
| Codex user config | `~/.codex/config.toml` |
| Copilot CLI binary | `/opt/homebrew/bin/copilot` |
| Copilot CLI source (minified) | `/opt/homebrew/Caskroom/copilot-cli/0.0.417/pkg/darwin-arm64/0.0.417/index.js` |
| Copilot config | `~/.copilot/config.json` |
| Copilot MCP config | `~/.copilot/mcp-config.json` |
| Copilot sessions | `~/.copilot/session-state/` |
| Copilot GitHub user | `Kangaxx-0` (already authenticated) |

### Important notes

- The login popup that appears when launching Copilot CLI is from the **Power BI MCP server** (`powerbi-remote` in `~/.copilot/mcp-config.json`), NOT from Copilot's own auth. Copilot's GitHub auth is cached silently.
- Codex **removed** Chat Completions wire format support - it only speaks Responses API now.
- The proxy approach avoids forking/modifying Codex's Rust code until we're confident in feasibility.

---

## Phase 1: Network Reconnaissance

### Task 1: Install mitmproxy

**Files:**
- No files modified

**Step 1: Install mitmproxy via Homebrew**

Run:
```bash
brew install mitmproxy
```

Expected: mitmproxy installed successfully. Verify with:
```bash
mitmproxy --version
```
Expected output: `Mitmproxy: 11.x.x` (any recent version)

---

### Task 2: Capture Copilot CLI network traffic

**Files:**
- Create: `codex-rs/tools/copilot-proxy/recon/capture-notes.md` (findings)

**Step 1: Start mitmproxy in one terminal**

Run in Terminal 1:
```bash
mkdir -p /Users/gaxx/Github/codex/codex-rs/tools/copilot-proxy/recon
cd /Users/gaxx/Github/codex/codex-rs/tools/copilot-proxy/recon
mitmproxy --listen-port 8080 --save-stream-file copilot-capture.flow
```

Expected: mitmproxy TUI starts, listening on port 8080.

**Step 2: Launch Copilot CLI through the proxy**

Run in Terminal 2:
```bash
cd /Users/gaxx/Work/MSS_AI_BUILD
HTTPS_PROXY=http://localhost:8080 \
NODE_TLS_REJECT_UNAUTHORIZED=0 \
copilot
```

Expected: Copilot CLI launches. If the Power BI MCP OAuth popup appears, dismiss it - we don't need that MCP for this test.

> **If Copilot CLI ignores the proxy** (no traffic appears in mitmproxy), try these fallbacks in order:
>
> Fallback A - Check Copilot's own proxy config:
> ```bash
> cat ~/.copilot/config.json | grep -i proxy
> ```
>
> Fallback B - Node.js debug logging:
> ```bash
> NODE_DEBUG=http,https HTTPS_PROXY=http://localhost:8080 NODE_TLS_REJECT_UNAUTHORIZED=0 copilot 2>copilot-debug.log
> ```
>
> Fallback C - macOS system-level proxy:
> Open System Settings > Network > Wi-Fi > Details > Proxies.
> Set "Secure Web Proxy (HTTPS)" to `localhost:8080`.
> Then launch `copilot` normally (no env vars needed).
> **Remember to disable this after capture.**
>
> Fallback D - Patch the bundled JS:
> The main bundle is at `/opt/homebrew/Caskroom/copilot-cli/0.0.417/pkg/darwin-arm64/0.0.417/index.js`.
> Back it up, then inject `console.log` at fetch/http.request calls.

**Step 3: Select GPT-5.x-Codex model and send a prompt**

In Copilot CLI:
1. Switch model to **GPT-5.3-Codex** (or any GPT-5.x-Codex variant)
2. Send prompt: `explain what 1+1 is`
3. Wait for the full response to complete
4. Optionally send a second prompt: `now explain 2+2` (to capture multi-turn)

**Step 4: Analyze captured traffic in mitmproxy**

In the mitmproxy TUI (Terminal 1), press `Enter` on each flow to inspect it.
Use `Tab` to switch between Request and Response views.

Record the following in `capture-notes.md`:

---

### Task 3: Document the auth token exchange

**Files:**
- Modify: `codex-rs/tools/copilot-proxy/recon/capture-notes.md`

**Step 1: Find the token exchange request**

In mitmproxy, look for a request early in the session (before any LLM call) that looks like a token exchange. Expected pattern:

```
POST https://api.github.com/copilot_internal/v2/token
```

or similar. It will likely have:
- An `Authorization: token ghp_xxxxx` header (GitHub OAuth token)
- A JSON response containing a `token` field and `expires_at` field

**Step 2: Record token exchange details in capture-notes.md**

Write to `codex-rs/tools/copilot-proxy/recon/capture-notes.md`:

```markdown
# Copilot Network Capture Findings

## Date: YYYY-MM-DD

## 1. Token Exchange

- **Endpoint:** [full URL]
- **Method:** [POST/GET]
- **Request headers:**
  - Authorization: [format, e.g., "token ghp_xxx"]
  - [any other headers]
- **Request body:** [if any]
- **Response status:** [200, etc.]
- **Response body (redacted):**
  ```json
  {
    "token": "[REDACTED - note format: tid=xxx? jwt? opaque?]",
    "expires_at": "[timestamp or ISO date]",
    "[other fields]": "..."
  }
  ```
- **Token lifetime:** [calculated from expires_at]
- **Source of GitHub token:** [gh auth token? keychain? copilot config?]
```

---

### Task 4: Document the LLM API endpoint and wire format

**Files:**
- Modify: `codex-rs/tools/copilot-proxy/recon/capture-notes.md`

**Step 1: Find the LLM API call**

In mitmproxy, find the request that carries the actual prompt. It will be the largest request/response flow, likely with `stream: true` and SSE response.

**Step 2: Record LLM API details in capture-notes.md**

Append to `capture-notes.md`:

```markdown
## 2. LLM API Endpoint

- **URL:** [full URL, e.g., https://api.enterprise.githubcopilot.com/v1/...]
- **Method:** POST
- **Wire format:** [Responses API / Chat Completions / Other]

### Request Headers (ALL of them):
- Authorization: Bearer [token from step 1]
- Content-Type: [application/json?]
- [List every header, e.g.:]
- X-GitHub-Api-Version: [value]
- Copilot-Integration-Id: [value]
- Editor-Version: [value]
- User-Agent: [value]
- OpenAI-Intent: [value]
- [etc.]

### Request Body (redacted):
```json
[paste full request body, redact any tokens]
```

### Response Format:
- Content-Type: [text/event-stream?]
- SSE event format:
```
[paste 3-5 example SSE events from the stream]
```

### Model Name in Request:
- Model string used: [e.g., "gpt-5.3-codex" or "gpt-5.3-codex-2026"]
```

---

### Task 5: Feasibility assessment

**Files:**
- Modify: `codex-rs/tools/copilot-proxy/recon/capture-notes.md`

**Step 1: Analyze findings and write assessment**

Append to `capture-notes.md`:

```markdown
## 3. Feasibility Assessment

### Wire Format Match
- [ ] Responses API (BEST CASE - proxy is trivial)
- [ ] Chat Completions (MEDIUM - translation needed, ~300-500 lines)
- [ ] Proprietary/unknown (HARD - significant reverse engineering needed)

### Auth Complexity
- [ ] Simple bearer token (can extract via gh auth token or similar)
- [ ] Multi-step token exchange (need to implement refresh logic)
- [ ] Complex/encrypted (may need to extract from Copilot's keychain)

### Required Headers
- [ ] Standard only (Authorization, Content-Type)
- [ ] Copilot-specific headers needed (list them)
- [ ] Editor/client fingerprinting required (may need to spoof)

### Blocking Issues
- [ ] None found - proceed to Phase 2
- [ ] [describe any blocking issue]

### Proxy Variant Decision
- [ ] Variant A: Thin proxy (Responses API match)
- [ ] Variant B: Translation proxy (Chat Completions)

### GO / NO-GO for Phase 2: [GO / NO-GO]
- Reason: [explain]
```

**Step 2: Commit findings**

Run:
```bash
cd /Users/gaxx/Github/codex
git add codex-rs/tools/copilot-proxy/recon/
git commit -m "docs: add copilot network capture findings from Phase 1 recon"
```

---

## Phase 2: Python Proxy Prototype

> **Prerequisites:** Phase 1 complete with GO decision. The exact implementation below will need to be adapted based on Phase 1 findings. The structure below covers both Variant A (thin proxy) and Variant B (translation proxy).

### Task 6: Set up the Python proxy project

**Files:**
- Create: `codex-rs/tools/copilot-proxy/proxy.py`
- Create: `codex-rs/tools/copilot-proxy/requirements.txt`
- Create: `codex-rs/tools/copilot-proxy/README.md`

**Step 1: Create requirements.txt**

Create `codex-rs/tools/copilot-proxy/requirements.txt`:
```
httpx[http2]>=0.27
aiohttp>=3.9
```

**Step 2: Set up virtual environment and install**

Run:
```bash
cd /Users/gaxx/Github/codex/codex-rs/tools/copilot-proxy
python3 -m venv .venv
source .venv/bin/activate
pip install -r requirements.txt
```

Expected: All packages install successfully.

**Step 3: Create the proxy skeleton**

Create `codex-rs/tools/copilot-proxy/proxy.py`:

```python
"""
Copilot API Proxy for Codex CLI.

Sits between Codex CLI and GitHub Copilot's LLM backend.
Handles auth token management and (optionally) wire format translation.

Usage:
    python proxy.py [--port 8888]

Then configure Codex:
    [model_providers.copilot]
    name = "GitHub Copilot"
    base_url = "http://localhost:8888"
    wire_api = "responses"
    requires_openai_auth = false
    supports_websockets = false
"""

import argparse
import asyncio
import json
import logging
import subprocess
import time
from typing import Optional

import httpx
from aiohttp import web

logging.basicConfig(level=logging.INFO, format="%(asctime)s %(levelname)s %(message)s")
logger = logging.getLogger(__name__)

# =============================================================================
# CONFIGURATION - Update these after Phase 1 recon
# =============================================================================

# Token exchange endpoint (from Phase 1, Task 3)
TOKEN_EXCHANGE_URL = "https://api.github.com/copilot_internal/v2/token"  # VERIFY

# LLM API endpoint (from Phase 1, Task 4)
COPILOT_LLM_URL = "https://FILL_FROM_PHASE1/v1/responses"  # VERIFY

# Required headers beyond Authorization (from Phase 1, Task 4)
COPILOT_EXTRA_HEADERS = {
    # "Copilot-Integration-Id": "FILL_FROM_PHASE1",
    # "Editor-Version": "FILL_FROM_PHASE1",
    # Add all required headers discovered in Phase 1
}

# Wire format: "responses" or "completions" (from Phase 1, Task 5)
WIRE_FORMAT = "responses"  # VERIFY - if "completions", translation code activates


# =============================================================================
# AUTH TOKEN MANAGEMENT
# =============================================================================

class CopilotTokenManager:
    """Manages the GitHub -> Copilot API token exchange and refresh."""

    def __init__(self):
        self.api_token: Optional[str] = None
        self.expires_at: float = 0

    def _get_github_token(self) -> str:
        """Get GitHub OAuth token from gh CLI."""
        result = subprocess.run(
            ["gh", "auth", "token"],
            capture_output=True,
            text=True,
            check=True,
        )
        token = result.stdout.strip()
        if not token:
            raise RuntimeError("No GitHub token found. Run 'gh auth login' first.")
        return token

    async def get_token(self) -> str:
        """Get a valid Copilot API token, refreshing if needed."""
        # Return cached token if still valid (with 60s buffer)
        if self.api_token and self.expires_at > time.time() + 60:
            return self.api_token

        logger.info("Refreshing Copilot API token...")
        github_token = self._get_github_token()

        async with httpx.AsyncClient() as client:
            resp = await client.get(
                TOKEN_EXCHANGE_URL,
                headers={
                    "Authorization": f"token {github_token}",
                    # Add any other required headers from Phase 1
                },
            )
            resp.raise_for_status()
            data = resp.json()

        self.api_token = data["token"]
        # Handle expires_at - could be unix timestamp or ISO date
        expires = data.get("expires_at", 0)
        if isinstance(expires, str):
            from datetime import datetime
            self.expires_at = datetime.fromisoformat(expires).timestamp()
        else:
            self.expires_at = float(expires)

        logger.info(f"Token refreshed, expires in {self.expires_at - time.time():.0f}s")
        return self.api_token


# =============================================================================
# PROXY SERVER
# =============================================================================

token_manager = CopilotTokenManager()


async def handle_responses(request: web.Request) -> web.StreamResponse:
    """
    Proxy handler for /v1/responses endpoint.

    Receives Codex's Responses API request, injects Copilot auth,
    and forwards to Copilot's backend.
    """
    body = await request.read()

    # Get fresh Copilot token
    copilot_token = await token_manager.get_token()

    # Build headers for Copilot backend
    headers = {
        "Authorization": f"Bearer {copilot_token}",
        "Content-Type": "application/json",
        **COPILOT_EXTRA_HEADERS,
    }

    if WIRE_FORMAT == "responses":
        # Variant A: Pass through unchanged
        forward_body = body
        forward_url = COPILOT_LLM_URL
    else:
        # Variant B: Translate Responses API -> Chat Completions
        forward_body = translate_request(json.loads(body))
        forward_url = COPILOT_LLM_URL  # Should point to /v1/chat/completions

    # Check if streaming
    request_data = json.loads(body)
    is_streaming = request_data.get("stream", True)

    async with httpx.AsyncClient(timeout=httpx.Timeout(300.0)) as client:
        if is_streaming:
            return await _stream_proxy(client, forward_url, headers, forward_body, request)
        else:
            resp = await client.post(forward_url, headers=headers, content=forward_body)
            response_body = resp.content
            if WIRE_FORMAT != "responses":
                response_body = translate_response(resp.json())
            return web.Response(
                status=resp.status_code,
                body=response_body,
                content_type="application/json",
            )


async def _stream_proxy(
    client: httpx.AsyncClient,
    url: str,
    headers: dict,
    body: bytes,
    original_request: web.Request,
) -> web.StreamResponse:
    """Stream SSE response from Copilot back to Codex."""
    response = web.StreamResponse(
        status=200,
        headers={"Content-Type": "text/event-stream", "Cache-Control": "no-cache"},
    )
    await response.prepare(original_request)

    async with client.stream("POST", url, headers=headers, content=body) as stream:
        async for line in stream.aiter_lines():
            if WIRE_FORMAT != "responses":
                line = translate_sse_event(line)
            await response.write(f"{line}\n".encode())

    await response.write_eof()
    return response


# =============================================================================
# WIRE FORMAT TRANSLATION (only used if WIRE_FORMAT == "completions")
# =============================================================================

def translate_request(responses_req: dict) -> bytes:
    """
    Translate Responses API request -> Chat Completions request.

    Only called if Phase 1 reveals Copilot uses Chat Completions.
    Adapt based on actual format observed.
    """
    # TODO: Implement based on Phase 1 findings
    # Basic mapping:
    messages = []
    for item in responses_req.get("input", []):
        role = item.get("role", "user")
        content = item.get("content", "")
        if isinstance(content, list):
            # Handle structured content (text, images, etc.)
            text_parts = [p["text"] for p in content if p.get("type") == "input_text"]
            content = "\n".join(text_parts)
        messages.append({"role": role, "content": content})

    completions_req = {
        "model": responses_req.get("model"),
        "messages": messages,
        "stream": responses_req.get("stream", True),
    }

    # Pass through common params
    for key in ["temperature", "max_tokens", "top_p", "tools", "tool_choice"]:
        if key in responses_req:
            completions_req[key] = responses_req[key]

    return json.dumps(completions_req).encode()


def translate_response(completions_resp: dict) -> bytes:
    """Translate Chat Completions response -> Responses API response."""
    # TODO: Implement based on Phase 1 findings
    raise NotImplementedError("Non-streaming translation not yet implemented")


def translate_sse_event(line: str) -> str:
    """
    Translate a single SSE event line from Chat Completions -> Responses API.

    Only called if Phase 1 reveals Copilot uses Chat Completions.
    Adapt based on actual SSE format observed.
    """
    # TODO: Implement based on Phase 1 findings
    # This is the most complex part - SSE event mapping
    # For now, pass through unchanged
    return line


# =============================================================================
# HEALTH & DIAGNOSTICS
# =============================================================================

async def handle_health(request: web.Request) -> web.Response:
    """Health check endpoint."""
    try:
        token = await token_manager.get_token()
        return web.json_response({
            "status": "ok",
            "token_valid": True,
            "token_expires_in": token_manager.expires_at - time.time(),
        })
    except Exception as e:
        return web.json_response(
            {"status": "error", "message": str(e)},
            status=500,
        )


async def handle_models(request: web.Request) -> web.Response:
    """Fake /v1/models endpoint for Codex compatibility."""
    return web.json_response({
        "data": [
            {"id": "gpt-5.3-codex", "object": "model"},
            {"id": "gpt-5.2-codex", "object": "model"},
            {"id": "gpt-5.1-codex", "object": "model"},
        ]
    })


# =============================================================================
# MAIN
# =============================================================================

def create_app() -> web.Application:
    app = web.Application()
    app.router.add_post("/v1/responses", handle_responses)
    app.router.add_get("/v1/models", handle_models)
    app.router.add_get("/health", handle_health)
    return app


def main():
    parser = argparse.ArgumentParser(description="Copilot API Proxy for Codex CLI")
    parser.add_argument("--port", type=int, default=8888, help="Port to listen on")
    parser.add_argument("--host", type=str, default="127.0.0.1", help="Host to bind to")
    args = parser.parse_args()

    logger.info(f"Starting Copilot proxy on {args.host}:{args.port}")
    logger.info(f"Wire format: {WIRE_FORMAT}")
    logger.info(f"Target: {COPILOT_LLM_URL}")

    app = create_app()
    web.run_app(app, host=args.host, port=args.port)


if __name__ == "__main__":
    main()
```

**Step 4: Create README**

Create `codex-rs/tools/copilot-proxy/README.md`:

```markdown
# Copilot API Proxy for Codex CLI

Routes Codex CLI's API calls through GitHub Copilot's backend.

## Setup

```bash
python3 -m venv .venv
source .venv/bin/activate
pip install -r requirements.txt
```

## Prerequisites

- GitHub CLI (`gh`) installed and authenticated (`gh auth login`)
- GitHub Copilot subscription (Business/Enterprise)
- Phase 1 recon completed (update constants in proxy.py)

## Usage

```bash
# Start the proxy
python proxy.py --port 8888

# In another terminal, use Codex with the Copilot backend
codex --provider copilot --model gpt-5.3-codex
```

## Codex Configuration

Add to `~/.codex/config.toml`:

```toml
[model_providers.copilot]
name = "GitHub Copilot"
base_url = "http://localhost:8888"
wire_api = "responses"
requires_openai_auth = false
supports_websockets = false
```
```

**Step 5: Commit**

Run:
```bash
cd /Users/gaxx/Github/codex
git add codex-rs/tools/copilot-proxy/
git commit -m "feat: add copilot proxy skeleton for Phase 2"
```

---

### Task 7: Update proxy constants from Phase 1 findings

**Files:**
- Modify: `codex-rs/tools/copilot-proxy/proxy.py` (top-level constants)

**Step 1: Open capture-notes.md and proxy.py side by side**

Read `codex-rs/tools/copilot-proxy/recon/capture-notes.md` and update these
constants in `proxy.py`:

- `TOKEN_EXCHANGE_URL` - from Task 3 findings
- `COPILOT_LLM_URL` - from Task 4 findings
- `COPILOT_EXTRA_HEADERS` - from Task 4 findings (all required headers)
- `WIRE_FORMAT` - from Task 5 assessment ("responses" or "completions")

**Step 2: If WIRE_FORMAT is "completions", implement translation functions**

Update `translate_request()`, `translate_response()`, and
`translate_sse_event()` based on the actual request/response formats
captured in Phase 1.

**Step 3: Verify token exchange method**

Check if the token exchange is GET or POST, and update `get_token()` in
`CopilotTokenManager` accordingly. Also verify:
- Is the GitHub token sent as `token ghp_xxx` or `Bearer ghp_xxx`?
- Are there additional required headers or query params?
- What's the exact response shape?

**Step 4: Commit**

Run:
```bash
cd /Users/gaxx/Github/codex
git add codex-rs/tools/copilot-proxy/proxy.py
git commit -m "feat: update proxy with Phase 1 recon findings"
```

---

### Task 8: Configure Codex for the Copilot provider

**Files:**
- Modify: `~/.codex/config.toml`

**Step 1: Add the copilot provider to Codex config**

Add to `~/.codex/config.toml` (create if it doesn't exist):

```toml
[model_providers.copilot]
name = "GitHub Copilot"
base_url = "http://localhost:8888"
wire_api = "responses"
requires_openai_auth = false
supports_websockets = false
```

**Step 2: Verify Codex recognizes the provider**

Run:
```bash
codex --provider copilot --model gpt-5.3-codex --help
```

Expected: Codex starts without errors about unknown provider.

---

### Task 9: Smoke test - simple prompt

**Files:**
- No files modified (testing only)

**Step 1: Start the proxy**

Run in Terminal 1:
```bash
cd /Users/gaxx/Github/codex/codex-rs/tools/copilot-proxy
source .venv/bin/activate
python proxy.py --port 8888
```

Expected: `Starting Copilot proxy on 127.0.0.1:8888`

**Step 2: Test health endpoint**

Run in Terminal 2:
```bash
curl http://localhost:8888/health
```

Expected: `{"status": "ok", "token_valid": true, "token_expires_in": ...}`

If this fails, the token exchange isn't working. Debug by checking:
- Is `gh auth token` returning a valid token?
- Is `TOKEN_EXCHANGE_URL` correct?
- Check proxy logs for error details.

**Step 3: Run Codex with a simple prompt**

Run in Terminal 2:
```bash
cd /Users/gaxx/Work/MSS_AI_BUILD
codex --provider copilot --model gpt-5.3-codex "explain what 1+1 is"
```

Expected: Codex sends request to proxy, proxy forwards to Copilot, response
streams back through Codex's TUI.

**If it fails:** Check proxy logs. Common issues:
- 401/403: Auth token invalid or expired. Verify token exchange.
- 400: Wire format mismatch. Compare request body to Phase 1 capture.
- Connection refused: Proxy not running or wrong port.
- Timeout: Copilot endpoint URL wrong.

---

### Task 10: Streaming test

**Files:**
- No files modified (testing only)

**Step 1: Verify streaming works**

Run Codex interactively:
```bash
codex --provider copilot --model gpt-5.3-codex
```

Send prompt: `write a python function that calculates fibonacci numbers, explain each line`

**Verify:**
- [ ] Response appears incrementally (not all at once)
- [ ] No truncation or garbled output
- [ ] Response completes without error

---

### Task 11: Multi-turn conversation test

**Files:**
- No files modified (testing only)

**Step 1: Test context retention**

In an interactive Codex session (from Task 10):

Turn 1: `define a variable x = 42`
Turn 2: `what is x?`

**Verify:**
- [ ] Second response references x = 42 correctly
- [ ] Conversation context maintained across turns

---

### Task 12: Tool use test

**Files:**
- No files modified (testing only)

**Step 1: Test Codex tool calls through the proxy**

In an interactive Codex session:

Prompt: `list the files in the current directory`

**Verify:**
- [ ] Codex sends tool call (shell exec) through the proxy
- [ ] Tool results are sent back to the model
- [ ] Model responds with file listing

Prompt: `read the first 5 lines of README.md`

**Verify:**
- [ ] File read tool call works
- [ ] Model sees and summarizes file content

---

### Task 13: Write Phase 2 findings and go/no-go for Phase 3

**Files:**
- Create: `codex-rs/tools/copilot-proxy/recon/phase2-findings.md`

**Step 1: Document Phase 2 results**

Write to `codex-rs/tools/copilot-proxy/recon/phase2-findings.md`:

```markdown
# Phase 2 Findings

## Date: YYYY-MM-DD

## Test Results

| Test | Status | Notes |
|------|--------|-------|
| Health check | PASS/FAIL | |
| Simple prompt | PASS/FAIL | |
| Streaming | PASS/FAIL | |
| Multi-turn | PASS/FAIL | |
| Tool use | PASS/FAIL | |

## Performance

- Proxy overhead latency: ~Xms
- Streaming behavior: [smooth / chunky / issues]
- Token refresh: [worked automatically / needed manual intervention]

## Issues Encountered

- [List any issues and workarounds]

## Wire Format Notes

- [Was translation needed? How complex?]
- [Any edge cases in the format mapping?]

## GO / NO-GO for Phase 3 (Rust Integration)

- [ ] GO - Integration works well, proxy overhead is acceptable,
      Rust integration would eliminate the proxy dependency
- [ ] NO-GO - [reason: too complex / unstable / not worth the effort]
- [ ] DEFER - Works but proxy is good enough, no need for Rust integration

## Phase 3 Scope Estimate (if GO)

- Auth module: [simple / moderate / complex]
- Wire format: [no translation / simple mapping / complex translation]
- Estimated effort: [small / medium / large]
```

**Step 2: Commit**

Run:
```bash
cd /Users/gaxx/Github/codex
git add codex-rs/tools/copilot-proxy/recon/phase2-findings.md
git commit -m "docs: add Phase 2 findings and Phase 3 go/no-go decision"
```

---

## Phase 3: Rust Integration (Deferred)

Design and tasks will be created after Phase 2 validates feasibility.
Scope depends on:
- Wire format discovered in Phase 1
- Translation complexity observed in Phase 2
- Auth token lifecycle and refresh requirements
- Any Copilot-specific quirks or limitations

---

## Known Risks

1. **Rate limiting:** Copilot may enforce stricter rate limits than direct
   OpenAI API access
2. **Header validation:** Copilot backend may reject requests without
   editor-specific headers (User-Agent spoofing may be needed)
3. **Wire format mismatch:** If Copilot uses a proprietary format, translation
   is harder
4. **Token exchange complexity:** May involve multiple steps or specific
   OAuth scopes
5. **TOS considerations:** Using Copilot's API outside official clients -
   review your org's policy
