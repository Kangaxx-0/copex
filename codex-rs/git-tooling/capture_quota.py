"""mitmproxy addon to capture Copilot quota and usage headers."""

from mitmproxy import http
import json
from datetime import datetime


class QuotaCapture:
    def response(self, flow: http.HTTPFlow):
        req = flow.request
        resp = flow.response
        if resp is None:
            return

        # Capture any response that has quota headers or is to a copilot/openai endpoint
        quota_headers = {}
        usage_data = None
        interesting = False

        for name, value in resp.headers.items(multi=True):
            lower = name.lower()
            if lower.startswith("x-quota-snapshot-"):
                quota_headers[name] = value
                interesting = True
            if lower.startswith("x-ratelimit"):
                quota_headers[name] = value
                interesting = True

        # Check if this is a chat completion or copilot endpoint
        url = req.pretty_url
        is_api_call = any(k in url for k in ["completions", "copilot", "openai"])

        if is_api_call or interesting:
            ts = datetime.now().strftime("%H:%M:%S")
            print(f"\n{'='*80}")
            print(f"[{ts}] {req.method} {url}")
            print(f"Status: {resp.status_code}")

            if quota_headers:
                print(f"\n--- Quota/Rate-Limit Headers ---")
                for k, v in sorted(quota_headers.items()):
                    print(f"  {k}: {v}")
            else:
                print(f"\n  (no quota headers in this response)")

            # Try to extract usage from JSON response body (for non-SSE)
            content_type = resp.headers.get("content-type", "")
            if "json" in content_type:
                try:
                    body = json.loads(resp.get_text())
                    if "usage" in body:
                        print(f"\n--- Usage from Response Body ---")
                        print(f"  {json.dumps(body['usage'], indent=2)}")
                except Exception:
                    pass

            print(f"{'='*80}")


addons = [QuotaCapture()]
