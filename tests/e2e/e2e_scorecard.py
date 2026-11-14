#!/usr/bin/env python3
"""
/w18 Track M Post-Migration Parity & E2E Acceptance Scorecard
Target: lnc-styles-gallery v1.0.0.1 (Rust/Axum single binary + React 19 SPA)
Stdlib-only Python 3 harness — zero third-party packages required.
"""

from __future__ import annotations

import base64
import hashlib
import http.client
import http.server
import json
import os
import platform
import shutil
import socket
import subprocess
import sys
import tempfile
import threading
import time
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[2]
EVIDENCE_DIR = REPO_ROOT / ".docs" / "test_evidence" / "2026-11-14-rust-v1"

HEX_MASTER_KEY = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f"
MGMT_KEY = "styles-mgmt-breakglass-2026-secret"
LEGACY_PLAINTEXT_TOKEN = "styles_LegacyPlaintextKey1234567890"

# Offline test-only RSA-2048 PKCS#8 private key (never used outside local test harness)
TEST_RSA_PEM = """-----BEGIN PRIVATE KEY-----
MIIEvQIBADANBgkqhkiG9w0BAQEFAASCBKcwggSjAgEAAoIBAQDanCl3dKzLmtqp
vp9kQqbgVkWSdv81I6UxfnNUArMePza6hEiK9gIrqwNZEtC9azOemGvkEIxpMNvD
garbakIv5NWYhP+V/+1QuehNypWjsKC1LZKtSEzizsTyMHyE7Ap2hWvmLXi08kBL
iCl2CsSUeK2abtHyD1BzXsaewykSVU8qXTA5TG18cmp8khFhhlYAnB6djPllhy4W
jMjTKPEXYyxOv2jgubUrTI8bfiKP3L/9DivP/4McdiAb/ZI85c2ZEhiuhL2oY27F
2XK4TQ7n2ZkyFw4IvfNpmwRTqVM3KGXE5yj3CeGpYHZnnT7sgGM2XzeMEW+dpa9t
6D3WuLSLAgMBAAECggEAaRO4xsdZK0CSrBaJLLrw1VICfOUfxF4O0Bx0uMPXbsl5
t+tPpyOaZXz8zfrCRNnbZPbknSnrI8CtUWrpQi/wAm9ez8HVzDbIzg2eAI5IoPaM
OKngH9RpyHcUqUjvAUEnwlJdS7U9mNP8XZHkQ+3nEWp0H4/27/+2bs32cGdkPX/K
B/c6jElxYkVbIfCfPvmwtCoYdtkUCdT9RpcfspGyYYaqk/a2eR6Yz/leKSk7fId2
i0cTbLUX+/wlKAZQGfSl6RxuXrEm9RSLWfDrXsipj5Lu+veWp7CHaxKXeVE1NyLX
RJMirpcjIJbD/K+3uslui1OgJ9d0jEuO5G8iEACaaQKBgQD7WfArn+Q29dI4OQ18
DAngsuX6QmBAHOVAU9PuVOiYXMEs+hcuIEbT5y6YEebnHs9KwdBdt7O9l0IWFwqS
3+u1/j73Xbx8xXQJG+kNpoSNK9xGuQGfGNTA6nR1DZY84qGzuGwcRtEKfpuB78pY
QMXrfxlyj6FpO8hOhvIAvp69TQKBgQDepzR55W7EDAlFibXe08J/OCclrFLjRnZk
m/qORFhgsxUrIwI0ftt82xHMVdlH7RuCKcOQtRa9MqrNTZ1VKblEMrXgzfp0gWsh
kvyp45UgdrBdnZODstyI8aPN6DrObA+JxPIpMObu+wipgvfNF4cxhOr/hvgMAgy/
aG/JOvWtNwKBgQCGaBii61RZA2iPggqw66sYifGqunXHsuvwUyNADg129Bqkl3Nv
yfi19aiXzvLfYj1M54kppEMsPccsEtQnqC/eWP+IvYo+WkPrXw5nq0S0TOD0vJee
YOPb6UOvGD3ffv0jIppFIKVF7F/xwq08Nh7gCE4uUZnpsPsy6rxcI5K0/QKBgAMI
qjli+/Bl7ZZp+vVvUz4iiL4FrBd2tVRHn9FicwXWStqMnpISjHEWFxb+sHg/bdmH
Mif9n7KyRwuHHDa129VnD31hsSD3Rnttok26USThGSQdao4n2MNw73d1y59Wb5vD
+DzBctQczZBpvXCo+mmdUwpqT2TyfjP5e1R1njVfAoGAbnlF04R8W06khOZD9pN6
re/jMjDKZfcE7/th1veJlCqC3urowQz3JLMDhYF8f8civ2pKs55O+w3+ShGZQm7v
kihVJ6UI9STSxTcNWpU1wYdYYBKe8AhhyJ0Jbyg0Ez1CeHibH6Stk1x2P4Ay/Qwq
Vf8UC7ZUga203h6vN6bHdL0=
-----END PRIVATE KEY-----"""

TEST_RSA_N = "2pwpd3Ssy5raqb6fZEKm4FZFknb_NSOlMX5zVAKzHj82uoRIivYCK6sDWRLQvWsznphr5BCMaTDbw4Gq22pCL-TVmIT_lf_tULnoTcqVo7CgtS2SrUhM4s7E8jB8hOwKdoVr5i14tPJAS4gpdgrElHitmm7R8g9Qc17GnsMpElVPKl0wOUxtfHJqfJIRYYZWAJwenYz5ZYcuFozI0yjxF2MsTr9o4Lm1K0yPG34ij9y__Q4rz_-DHHYgG_2SPOXNmRIYroS9qGNuxdlyuE0O59mZMhcOCL3zaZsEU6lTNyhlxOco9wnhqWB2Z50-7IBjNl83jBFvnaWvbeg91ri0iw"
TEST_RSA_E = "AQAB"


def b64u(data: bytes) -> str:
    return base64.urlsafe_b64encode(data).rstrip(b"=").decode("ascii")


def _der_read_tlv(buf: bytes, idx: int) -> tuple[int, bytes, int]:
    tag = buf[idx]
    idx += 1
    first = buf[idx]
    idx += 1
    if first < 0x80:
        length = first
    else:
        num_bytes = first & 0x7F
        length = int.from_bytes(buf[idx : idx + num_bytes], "big")
        idx += num_bytes
    val = buf[idx : idx + length]
    return tag, val, idx + length


def _extract_rsa_private_numbers(pem_text: str) -> tuple[int, int]:
    b64_lines = [
        ln.strip()
        for ln in pem_text.strip().splitlines()
        if not ln.startswith("-----")
    ]
    der = base64.b64decode("".join(b64_lines))
    _, pkcs8_seq, _ = _der_read_tlv(der, 0)
    _, _ver, p = _der_read_tlv(pkcs8_seq, 0)
    _, _alg, p = _der_read_tlv(pkcs8_seq, p)
    _, pkcs1_octets, _ = _der_read_tlv(pkcs8_seq, p)
    _, rsa_seq, _ = _der_read_tlv(pkcs1_octets, 0)
    _, _rver, q = _der_read_tlv(rsa_seq, 0)
    _, mod_bytes, q = _der_read_tlv(rsa_seq, q)
    _, _pub_e, q = _der_read_tlv(rsa_seq, q)
    _, priv_d_bytes, _ = _der_read_tlv(rsa_seq, q)
    n = int.from_bytes(mod_bytes, "big")
    d = int.from_bytes(priv_d_bytes, "big")
    return n, d


RSA_N_INT, RSA_D_INT = _extract_rsa_private_numbers(TEST_RSA_PEM)


def sign_rs256_jwt(claims: dict[str, Any], kid: str = "test-rs256-k1") -> str:
    header = {"alg": "RS256", "typ": "JWT", "kid": kid}
    signing_input = (
        f"{b64u(json.dumps(header, separators=(',', ':')).encode())}."
        f"{b64u(json.dumps(claims, separators=(',', ':')).encode())}"
    ).encode("ascii")
    # DigestInfo prefix for SHA-256 (RFC 8017 §9.2)
    digest_info = (
        bytes.fromhex("3031300d060960864801650304020105000420")
        + hashlib.sha256(signing_input).digest()
    )
    k = 256
    ps_len = k - len(digest_info) - 3
    em = b"\x00\x01" + (b"\xff" * ps_len) + b"\x00" + digest_info
    sig_int = pow(int.from_bytes(em, "big"), RSA_D_INT, RSA_N_INT)
    sig_bytes = sig_int.to_bytes(k, "big")
    return f"{signing_input.decode('ascii')}.{b64u(sig_bytes)}"


def find_free_port() -> int:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.bind(("127.0.0.1", 0))
        return int(s.getsockname()[1])


class MockOidcIdp:
    """Minimal local Keycloak OIDC IdP implementing discovery, JWKS, authorize, and PKCE S256 token exchange."""

    def __init__(self, port: int) -> None:
        self.port = port
        self.issuer = f"http://127.0.0.1:{port}/realms/lengoc"
        self.codes: dict[str, dict[str, Any]] = {}
        self.next_username = "louis"
        self._server = http.server.ThreadingHTTPServer(
            ("127.0.0.1", port), self._make_handler()
        )
        self._thread = threading.Thread(target=self._server.serve_forever, daemon=True)

    def start(self) -> None:
        self._thread.start()

    def stop(self) -> None:
        self._server.shutdown()
        self._server.server_close()

    def _make_handler(self):
        idp = self

        class Handler(http.server.BaseHTTPRequestHandler):
            def log_message(self, fmt: str, *args: Any) -> None:
                return

            def _send_json(self, code: int, data: Any) -> None:
                raw = json.dumps(data).encode("utf-8")
                self.send_response(code)
                self.send_header("Content-Type", "application/json")
                self.send_header("Content-Length", str(len(raw)))
                self.end_headers()
                self.wfile.write(raw)

            def do_GET(self) -> None:
                parsed = urllib.parse.urlsplit(self.path)
                if parsed.path == "/realms/lengoc/.well-known/openid-configuration":
                    self._send_json(
                        200,
                        {
                            "issuer": idp.issuer,
                            "authorization_endpoint": f"{idp.issuer}/protocol/openid-connect/auth",
                            "token_endpoint": f"{idp.issuer}/protocol/openid-connect/token",
                            "jwks_uri": f"{idp.issuer}/protocol/openid-connect/certs",
                        },
                    )
                elif parsed.path == "/realms/lengoc/protocol/openid-connect/certs":
                    self._send_json(
                        200,
                        {
                            "keys": [
                                {
                                    "kty": "RSA",
                                    "alg": "RS256",
                                    "use": "sig",
                                    "kid": "test-rs256-k1",
                                    "n": TEST_RSA_N,
                                    "e": TEST_RSA_E,
                                }
                            ]
                        },
                    )
                else:
                    self._send_json(404, {"error": "not_found"})

            def do_POST(self) -> None:
                parsed = urllib.parse.urlsplit(self.path)
                if parsed.path != "/realms/lengoc/protocol/openid-connect/token":
                    self._send_json(404, {"error": "not_found"})
                    return
                length = int(self.headers.get("Content-Length", "0"))
                body = self.rfile.read(length).decode("utf-8")
                form = dict(urllib.parse.parse_qsl(body))
                code = form.get("code", "")
                verifier = form.get("code_verifier", "")
                entry = idp.codes.pop(code, None)
                if not entry:
                    self._send_json(400, {"error": "invalid_grant"})
                    return
                computed_challenge = b64u(hashlib.sha256(verifier.encode("ascii")).digest())
                if computed_challenge != entry["code_challenge"]:
                    self._send_json(400, {"error": "pkce_mismatch"})
                    return
                now = int(time.time())
                uname = entry["username"]
                claims = {
                    "iss": idp.issuer,
                    "sub": f"kc-{uname}",
                    "aud": "lnc-styles-gallery",
                    "azp": "lnc-styles-gallery",
                    "exp": now + 300,
                    "iat": now,
                    "nonce": entry["nonce"],
                    "preferred_username": uname,
                    "email": f"{uname}@lengoc.me",
                    "name": uname.title(),
                    "groups": ["/admins"] if uname == "louis" else ["/guests"],
                }
                id_token = sign_rs256_jwt(claims)
                self._send_json(
                    200,
                    {
                        "access_token": "mock-access-token",
                        "token_type": "Bearer",
                        "expires_in": 300,
                        "id_token": id_token,
                    },
                )

        return Handler


def raw_http(
    port: int,
    method: str,
    path: str,
    headers: dict[str, str] | None = None,
    body: bytes | str | None = None,
) -> tuple[int, dict[str, str], list[str], bytes]:
    """Low-level HTTP request preserving raw paths (including `..`) and multiple Set-Cookie headers."""
    conn = http.client.HTTPConnection("127.0.0.1", port, timeout=10)
    hdrs = dict(headers or {})
    payload: bytes | None = None
    if isinstance(body, str):
        payload = body.encode("utf-8")
    elif isinstance(body, bytes):
        payload = body
    if payload is not None and "Content-Length" not in hdrs:
        hdrs["Content-Length"] = str(len(payload))
    conn.request(method, path, body=payload, headers=hdrs)
    resp = conn.getresponse()
    raw = resp.read()
    all_hdrs = {}
    set_cookies: list[str] = []
    for k, v in resp.getheaders():
        kl = k.lower()
        if kl == "set-cookie":
            set_cookies.append(v)
        all_hdrs[kl] = v
    conn.close()
    return resp.status, all_hdrs, set_cookies, raw


def build_fixture_corpus(data_dir: Path) -> None:
    """Create a realistic 3-corpus fixture inside `data_dir`."""
    refero = data_dir / "refero-styles"
    styles_sub = refero / "styles"
    (styles_sub / "s-obsidian-cli" / "shots").mkdir(parents=True)
    (styles_sub / "s-lnc-blueprint").mkdir(parents=True)
    (styles_sub / "s-fallback-only").mkdir(parents=True)
    (refero / "sitemaps").mkdir(parents=True)

    index_items = [
        {
            "id": "s-obsidian-cli",
            "title": "Obsidian Terminal Pro",
            "tagline": "Monospace developer dashboard & CLI console",
            "theme": "dark",
            "url": "https://styles.refero.design/styles/s-obsidian-cli",
            "source_url": "https://linear.app",
        },
        {
            "id": "s-lnc-blueprint",
            "title": "LNC Paper Sketch Blueprint",
            "tagline": "Warm minimal lengoc.me internal app shell",
            "theme": "light",
            "url": "https://styles.refero.design/styles/s-lnc-blueprint",
            "source_url": "https://styles.lengoc.me",
        },
        {
            "id": "s-fallback-only",
            "title": "Fintech Vault UI",
            "tagline": "High-trust payment ledger & crypto treasury",
            "theme": "dark",
            "url": "https://styles.refero.design/styles/s-fallback-only",
            "source_url": "https://stripe.com",
        },
    ]
    (refero / "index.json").write_text(json.dumps(index_items), encoding="utf-8")
    (styles_sub / "s-obsidian-cli" / "DESIGN.md").write_text(
        "# Obsidian Terminal Pro\nUse JetBrains Mono, #090d16 canvas, and crisp 1px borders.",
        encoding="utf-8",
    )
    (styles_sub / "s-obsidian-cli" / "meta.json").write_text(
        json.dumps({"palette": ["#090d16", "#10b981"]}), encoding="utf-8"
    )
    (styles_sub / "s-obsidian-cli" / "preview.svg").write_text(
        '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10"><rect width="10" height="10"/></svg>',
        encoding="utf-8",
    )
    (styles_sub / "s-obsidian-cli" / "shots" / "01.svg").write_text(
        '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 8 8"><circle cx="4" cy="4" r="3"/></svg>',
        encoding="utf-8",
    )
    (styles_sub / "s-lnc-blueprint" / "DESIGN.md").write_text(
        "# LNC Blueprint\nWarm editorial tokens with #5B2E63 primary accent.",
        encoding="utf-8",
    )
    (styles_sub / "s-lnc-blueprint" / "preview.svg").write_text(
        '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10"><rect width="10" height="10"/></svg>',
        encoding="utf-8",
    )
    (refero / "sitemaps" / "collections.xml").write_text(
        '<?xml version="1.0" encoding="UTF-8"?><urlset><url><loc>https://styles.refero.design/collections/obsidian-picks</loc></url></urlset>',
        encoding="utf-8",
    )

    # Prompt-Driven UI corpus (1 EN + 1 VI + sibling asset)
    pui = data_dir / "prompt-driven-ui"
    (pui / "websites-en").mkdir(parents=True)
    (pui / "prompts-en").mkdir(parents=True)
    (pui / "websites").mkdir(parents=True)
    (pui / "prompts").mkdir(parents=True)
    (pui / "thumbs").mkdir(parents=True)

    (pui / "websites-en" / "01-saas-landing.html").write_text(
        '<!doctype html><html><head><title>SaaS Copilot Studio</title><meta name="description" content="AI workflow landing page"><link rel="stylesheet" href="theme.css"></head><body><h1>SaaS Copilot</h1></body></html>',
        encoding="utf-8",
    )
    (pui / "websites-en" / "theme.css").write_text(
        "body { background: #0b1120; color: #f8fafc; }", encoding="utf-8"
    )
    (pui / "prompts-en" / "01-saas-landing.md").write_text(
        "Design a dark B2B AI SaaS landing page with pricing cards.", encoding="utf-8"
    )
    (pui / "thumbs" / "01-saas-landing.svg").write_text(
        '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 9"></svg>',
        encoding="utf-8",
    )

    (pui / "websites" / "02-coffee-roaster.html").write_text(
        "<!doctype html><html><head><title>Cà Phê Đặc Sản</title></head><body><h1>Rang Xay</h1></body></html>",
        encoding="utf-8",
    )
    (pui / "prompts" / "02-coffee-roaster.md").write_text(
        "Thiết kế trang chủ thương hiệu cà phê đặc sản Việt Nam.", encoding="utf-8"
    )

    # LNC & Knowns Components corpus (1 HTML + 1 TSX)
    comps = data_dir / "lnc-components"
    (comps / "items" / "lnc-neon-btn").mkdir(parents=True)
    (comps / "items" / "knowns-agent-plan").mkdir(parents=True)
    comp_index = [
        {
            "id": "lnc-neon-btn",
            "name": "LNC Neon Button",
            "category": "buttons",
            "kind": "html",
            "description": "Glowing interactive CTA button",
            "tags": ["button", "glow"],
            "license": "LNC Internal",
            "author": "LNC",
        },
        {
            "id": "knowns-agent-plan",
            "name": "Knowns Agent Plan Card",
            "category": "ai-agent",
            "kind": "tsx",
            "entry": "agent-plan.tsx",
            "description": "Step-by-step agent task checklist",
            "tags": ["agent", "plan"],
            "license": "MIT",
            "source_url": "https://ui.knowns.dev",
            "author": "Knowns Lab",
        },
    ]
    (comps / "index.json").write_text(json.dumps(comp_index), encoding="utf-8")
    (comps / "items" / "lnc-neon-btn" / "index.html").write_text(
        "<!doctype html><html><body><button class='glow'>Launch</button></body></html>",
        encoding="utf-8",
    )
    (comps / "items" / "knowns-agent-plan" / "agent-plan.tsx").write_text(
        "export function AgentPlan() { return <div>Agent Plan</div> }",
        encoding="utf-8",
    )


def snapshot_tree(root: Path) -> dict[str, tuple[int, int]]:
    snap: dict[str, tuple[int, int]] = {}
    for p in sorted(root.rglob("*")):
        rel = p.relative_to(root).as_posix()
        st = p.stat()
        snap[rel] = (st.st_size, int(st.st_mtime_ns))
    return snap


def get_process_rss_mb(pid: int) -> float:
    try:
        if platform.system().lower().startswith("win"):
            out = subprocess.check_output(
                ["tasklist", "/FI", f"PID eq {pid}", "/FO", "CSV", "/NH"],
                text=True,
                timeout=5,
            ).strip()
            if out and "," in out:
                parts = [p.strip('"') for p in out.split('","')]
                mem_col = parts[-1]
                digits = "".join(ch for ch in mem_col if ch.isdigit())
                if digits:
                    return round(int(digits) / 1024.0, 2)
        else:
            out = subprocess.check_output(
                ["ps", "-o", "rss=", "-p", str(pid)], text=True, timeout=5
            ).strip()
            if out:
                return round(int(out) / 1024.0, 2)
    except Exception:
        pass
    return 0.0


class ScorecardRunner:
    def __init__(self) -> None:
        self.checks: list[dict[str, Any]] = []
        self.metrics: dict[str, Any] = {}

    def record(
        self,
        category: str,
        check_id: str,
        title: str,
        passed: bool,
        severity: str,
        detail: str,
    ) -> None:
        self.checks.append(
            {
                "category": category,
                "id": check_id,
                "title": title,
                "passed": bool(passed),
                "severity": severity,
                "detail": detail,
            }
        )
        mark = "PASS" if passed else f"FAIL[{severity}]"
        print(f"  [{mark}] {check_id} {title} — {detail}")


def extract_cookie(set_cookies: list[str], name: str) -> str | None:
    prefix = f"{name}="
    for sc in set_cookies:
        if sc.startswith(prefix):
            return sc.split(";", 1)[0]
    return None


def run_scorecard() -> int:
    runner = ScorecardRunner()

    exe_name = (
        "lnc-styles-gallery.exe"
        if platform.system().lower().startswith("win")
        else "lnc-styles-gallery"
    )
    bin_path = REPO_ROOT / "server-rs" / "target" / "release" / exe_name
    ui_dist = REPO_ROOT / "ui" / "dist"

    print("=== LNC Styles Gallery v1.0.0.1 — /w18 Track M E2E Scorecard ===")

    # Pre-check binary & UI dist
    bin_exists = bin_path.is_file() and bin_path.stat().st_size > 0
    bin_size = bin_path.stat().st_size if bin_exists else 0
    runner.metrics["binary_size_bytes"] = bin_size
    runner.record(
        "health",
        "H01",
        "Compiled release binary & built React SPA bundle present",
        bin_exists and (ui_dist / "index.html").is_file(),
        "Critical",
        f"binary={bin_size:,} bytes, ui/dist/index.html={(ui_dist / 'index.html').is_file()}",
    )
    if not bin_exists:
        print("FATAL: Release binary missing; build `cargo build --release` first.")
        return 2

    # Static scan of ui/dist for any leaked `raw_key` identifier
    dist_has_raw_key = False
    js_bundle_rel: str | None = None
    for f in ui_dist.rglob("*"):
        if f.is_file() and f.suffix in {".js", ".html", ".css"}:
            txt = f.read_text(encoding="utf-8", errors="ignore")
            if "raw_key" in txt:
                dist_has_raw_key = True
            if f.suffix == ".js" and js_bundle_rel is None:
                js_bundle_rel = "/" + f.relative_to(ui_dist).as_posix()
    runner.record(
        "security",
        "S01",
        "Compiled SPA bundle contains zero `raw_key` references",
        not dist_has_raw_key and js_bundle_rel is not None,
        "High",
        f"raw_key_in_dist={dist_has_raw_key}, sample_bundle={js_bundle_rel}",
    )

    idp_port = find_free_port()
    app_port = find_free_port()
    idp = MockOidcIdp(idp_port)
    idp.start()

    tmp_root = Path(tempfile.mkdtemp(prefix="styles-rs-e2e-"))
    proc: subprocess.Popen[bytes] | None = None

    try:
        data_dir = tmp_root / "data"
        cfg_dir = tmp_root / "config"
        build_fixture_corpus(data_dir)
        cfg_dir.mkdir(parents=True)

        # Seed legacy plaintext keys.json to verify auto-migration
        (cfg_dir / "keys.json").write_text(
            json.dumps(
                {
                    "api_keys": [
                        {
                            "id": "hub_default",
                            "label": "Business MCP Hub",
                            "key": LEGACY_PLAINTEXT_TOKEN,
                            "enabled": True,
                        }
                    ]
                }
            ),
            encoding="utf-8",
        )

        corpus_before = snapshot_tree(data_dir)

        env = os.environ.copy()
        env.update(
            {
                "PORT": str(app_port),
                "STYLES_HOST": "127.0.0.1",
                "STYLES_ROOT_DIR": str(REPO_ROOT),
                "STYLES_DATA_DIR": str(data_dir),
                "STYLES_CONFIG_DIR": str(cfg_dir),
                "STYLES_PUBLIC_URL": f"http://127.0.0.1:{app_port}",
                "LNC_MASTER_KEY": HEX_MASTER_KEY,
                "MANAGEMENT_KEY": MGMT_KEY,
                "STYLES_MCP_AUTH": "1",
                "OIDC_ISSUER_URL": idp.issuer,
                "OIDC_CLIENT_ID": "lnc-styles-gallery",
                "OIDC_CLIENT_SECRET": "mock-oidc-secret",
                "OIDC_REDIRECT_URI": f"http://127.0.0.1:{app_port}/api/auth/oidc/callback",
                "OIDC_ALLOWED_USERS": "louis",
                "OIDC_ALLOWED_GROUPS": "/admins",
            }
        )

        t_boot0 = time.perf_counter()
        proc = subprocess.Popen(
            [str(bin_path)],
            cwd=str(REPO_ROOT),
            env=env,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )

        # Wait until /health responds 200
        boot_ms = 0.0
        ready = False
        for _ in range(60):
            try:
                st, _, _, body = raw_http(app_port, "GET", "/health")
                if st == 200:
                    ready = True
                    boot_ms = round((time.perf_counter() - t_boot0) * 1000.0, 2)
                    break
            except Exception:
                time.sleep(0.05)

        runner.metrics["boot_ms"] = boot_ms
        idle_rss = get_process_rss_mb(proc.pid)
        runner.metrics["idle_rss_mb"] = idle_rss

        runner.record(
            "health",
            "H02",
            "Server boots cleanly on ephemeral PORT and serves /health",
            ready,
            "Critical",
            f"boot_ms={boot_ms}ms, idle_rss={idle_rss}MB",
        )
        if not ready:
            raise RuntimeError("Server failed to boot within 3s")

        # CLI --healthcheck flag
        hc = subprocess.run(
            [str(bin_path), "--healthcheck"],
            env=env,
            capture_output=True,
            timeout=5,
        )
        runner.record(
            "health",
            "H03",
            "Built-in `--healthcheck` subcommand exits 0",
            hc.returncode == 0,
            "Medium",
            f"exit_code={hc.returncode}",
        )

        # Latency benchmark (40 requests)
        latencies: list[float] = []
        health_json: dict[str, Any] = {}
        for i in range(40):
            path = "/health" if i % 2 == 0 else "/api/styles?limit=10"
            t0 = time.perf_counter()
            st, _, _, b = raw_http(app_port, "GET", path)
            latencies.append((time.perf_counter() - t0) * 1000.0)
            if i == 0 and st == 200:
                health_json = json.loads(b.decode("utf-8"))
        latencies.sort()
        p50 = round(latencies[len(latencies) // 2], 2)
        p95 = round(latencies[int(len(latencies) * 0.95)], 2)
        load_rss = get_process_rss_mb(proc.pid)
        runner.metrics.update({"p50_ms": p50, "p95_ms": p95, "post_load_rss_mb": load_rss})

        runner.record(
            "health",
            "H04",
            "Version 1.0.0.1, corpus counts, p95 latency (<50ms) & RSS (<80MB)",
            health_json.get("version") == "1.0.0.1"
            and health_json.get("styles_count") == 3
            and health_json.get("templates_count") == 2
            and health_json.get("components_count") == 2
            and p95 < 50.0
            and load_rss < 80.0,
            "High",
            f"v={health_json.get('version')}, styles={health_json.get('styles_count')}, p50={p50}ms, p95={p95}ms, rss={load_rss}MB",
        )

        # ------------------------------------------------------------------
        # Category 2: Core User Journeys & Corpus REST Parity (25%)
        # ------------------------------------------------------------------
        st, _, _, b = raw_http(app_port, "GET", "/api/styles?theme=dark")
        styles_dark = json.loads(b)
        st_d, _, _, b_d = raw_http(app_port, "GET", "/api/styles/s-obsidian-cli")
        detail_obs = json.loads(b_d)
        st_p1, h_p1, _, _ = raw_http(app_port, "GET", "/api/styles/s-obsidian-cli/preview")
        st_p2, h_p2, _, b_p2 = raw_http(app_port, "GET", "/api/styles/s-fallback-only/preview")
        st_sh, _, _, b_sh = raw_http(app_port, "GET", "/api/styles/s-obsidian-cli/shots")
        st_s1, _, _, _ = raw_http(app_port, "GET", "/api/styles/s-obsidian-cli/shots/1")
        runner.record(
            "journeys",
            "J01",
            "Styles list, filter, detail, preview SVG, fallback SVG & shots",
            st == 200
            and styles_dark["total"] == 2
            and st_d == 200
            and "JetBrains Mono" in detail_obs.get("design_md", "")
            and st_p1 == 200
            and "svg" in h_p1.get("content-type", "")
            and st_p2 == 200
            and b"Fintech Vault UI" in b_p2
            and st_sh == 200
            and len(json.loads(b_sh)) == 1
            and st_s1 == 200,
            "High",
            f"dark_total={styles_dark['total']}, fallback_svg_bytes={len(b_p2)}",
        )

        # Search & Collections
        st_sr, _, _, b_sr = raw_http(app_port, "GET", "/api/search?q=copilot")
        sr = json.loads(b_sr)
        st_sr2, _, _, b_sr2 = raw_http(app_port, "GET", "/api/search?q=jetbrains&include_body=true")
        sr2 = json.loads(b_sr2)
        st_cl, _, _, b_cl = raw_http(app_port, "GET", "/api/collections")
        cols = json.loads(b_cl)
        st_cd, _, _, b_cd = raw_http(app_port, "GET", "/api/collections/lnc-style")
        col_lnc = json.loads(b_cd)
        runner.record(
            "journeys",
            "J02",
            "Full-text search (styles + templates + components) & collections (incl. sitemap)",
            st_sr == 200
            and sr["total_templates"] == 1
            and st_sr2 == 200
            and sr2["total_styles"] == 1
            and st_cl == 200
            and any(c["id"] == "sitemap-obsidian-picks" for c in cols["items"])
            and st_cd == 200
            and col_lnc["count"] >= 1,
            "High",
            f"collections={cols['total']}, lnc_style_count={col_lnc.get('count')}",
        )

        # Templates (EN/VI, HTML demo CSP, thumb, prompt, sibling CSS asset)
        st_tl, _, _, b_tl = raw_http(app_port, "GET", "/api/templates?lang=vi")
        tpl_vi = json.loads(b_tl)
        st_td, _, _, b_td = raw_http(app_port, "GET", "/api/templates/en-01-saas-landing")
        tpl_d = json.loads(b_td)
        st_th, h_th, _, b_th = raw_http(app_port, "GET", "/api/templates/en-01-saas-landing/html")
        st_tt, _, _, _ = raw_http(app_port, "GET", "/api/templates/vi-02-coffee-roaster/thumb")
        st_tp, _, _, b_tp = raw_http(app_port, "GET", "/api/templates/en-01-saas-landing/prompt")
        st_ta, h_ta, _, b_ta = raw_http(app_port, "GET", "/api/templates/en-01-saas-landing/theme.css")
        runner.record(
            "journeys",
            "J03",
            "Templates EN/VI list, detail, iframe HTML, fallback thumb, prompt & sibling CSS",
            st_tl == 200
            and tpl_vi["total"] == 1
            and st_td == 200
            and "pricing cards" in tpl_d.get("prompt", "")
            and st_th == 200
            and b"SaaS Copilot" in b_th
            and "frame-ancestors 'self'" in h_th.get("content-security-policy", "")
            and st_tt == 200
            and st_tp == 200
            and b"pricing cards" in b_tp
            and st_ta == 200
            and "text/css" in h_ta.get("content-type", ""),
            "High",
            f"vi_total={tpl_vi['total']}, html_bytes={len(b_th)}, css_bytes={len(b_ta)}",
        )

        # Components (HTML + TSX + source + file download)
        st_cm, _, _, b_cm = raw_http(app_port, "GET", "/api/components")
        cm_list = json.loads(b_cm)
        st_cd1, _, _, b_cd1 = raw_http(
            app_port, "GET", "/api/components/knowns-agent-plan?include_source=true"
        )
        cm_tsx = json.loads(b_cd1)
        st_ch, _, _, b_ch = raw_http(app_port, "GET", "/api/components/lnc-neon-btn/html")
        st_cf, _, _, b_cf = raw_http(
            app_port, "GET", "/api/components/knowns-agent-plan/files/agent-plan.tsx"
        )
        runner.record(
            "journeys",
            "J04",
            "Components catalog (HTML + TSX), inline sources & file serving",
            st_cm == 200
            and cm_list["total"] == 2
            and st_cd1 == 200
            and "AgentPlan" in cm_tsx.get("sources", {}).get("agent-plan.tsx", "")
            and st_ch == 200
            and b"Launch" in b_ch
            and st_cf == 200
            and b"AgentPlan" in b_cf,
            "High",
            f"components_total={cm_list['total']}",
        )

        # Licenses, MCP info & SPA routes
        st_lic, _, _, b_lic = raw_http(app_port, "GET", "/api/licenses")
        lic = json.loads(b_lic)
        st_mi, _, _, b_mi = raw_http(app_port, "GET", "/api/mcp/info")
        mi = json.loads(b_mi)
        st_spa, h_spa, _, b_spa = raw_http(app_port, "GET", "/styles")
        st_js, h_js, _, _ = raw_http(app_port, "GET", js_bundle_rel or "/favicon.svg")
        st_404_asset, _, _, _ = raw_http(app_port, "GET", "/static/missing-chunk-999.js")
        runner.record(
            "journeys",
            "J05",
            "Licenses (4 corpora), /api/mcp/info, SPA HTML shell, hashed JS & missing-asset 404",
            st_lic == 200
            and len(lic.get("corpora", [])) == 4
            and st_mi == 200
            and mi.get("tools_count") == 8
            and st_spa == 200
            and b'<div id="root">' in b_spa
            and "script-src 'self'" in h_spa.get("content-security-policy", "")
            and st_js == 200
            and "javascript" in h_js.get("content-type", "")
            and st_404_asset == 404,
            "Medium",
            f"corpora={len(lic.get('corpora', []))}, mcp_tools={mi.get('tools_count')}",
        )

        # ------------------------------------------------------------------
        # Category 3: Security, Crypto, OIDC & Zero-F12-Leak Parity (30%)
        # ------------------------------------------------------------------
        disk_keys_text = (cfg_dir / "keys.json").read_text(encoding="utf-8")
        disk_cfg_text = (cfg_dir / "config.json").read_text(encoding="utf-8")
        runner.record(
            "security",
            "S02",
            "Legacy `keys.json` & `MANAGEMENT_KEY` scrubbed on disk (HMAC + Argon2id only)",
            LEGACY_PLAINTEXT_TOKEN not in disk_keys_text
            and '"raw_key"' not in disk_keys_text
            and '"key_hash"' in disk_keys_text
            and MGMT_KEY not in disk_cfg_text
            and "$argon2id$" in disk_cfg_text,
            "Critical",
            "keys.json holds salted HMAC digest; config.json holds Argon2id PHC hash",
        )

        # Unauthenticated gates on /keys, /admin, /api/keys, /api/admin/*, /mcp
        st_uk, h_uk, _, _ = raw_http(app_port, "GET", "/keys")
        st_ua, h_ua, _, _ = raw_http(app_port, "GET", "/admin")
        st_uak, _, _, _ = raw_http(app_port, "GET", "/api/keys")
        st_uar, _, _, _ = raw_http(app_port, "POST", "/api/admin/reindex")
        st_umcp, _, _, _ = raw_http(
            app_port,
            "POST",
            "/mcp",
            {"Content-Type": "application/json"},
            json.dumps({"jsonrpc": "2.0", "id": 1, "method": "tools/list"}),
        )
        runner.record(
            "security",
            "S03",
            "Unauthenticated requests blocked (302 on /keys & /admin; 401 on /api/keys, /api/admin/*, /mcp)",
            st_uk == 302
            and "/login.html" in h_uk.get("location", "")
            and st_ua == 302
            and "/login.html" in h_ua.get("location", "")
            and st_uak == 401
            and st_uar == 401
            and st_umcp == 401,
            "Critical",
            f"/keys={st_uk}, /admin={st_ua}, /api/keys={st_uak}, /mcp={st_umcp}",
        )

        # Login page structure + strict security headers & CSP
        st_lg, h_lg, _, b_lg = raw_http(app_port, "GET", "/login.html")
        lg_html = b_lg.decode("utf-8")
        csp_val = h_lg.get("content-security-policy", "")
        runner.record(
            "security",
            "S04",
            "/login.html OIDC SSO primary + collapsed `<details>` break-glass + strict headers/CSP",
            st_lg == 200
            and "Sign in with LNC SSO" in lg_html
            and "<details" in lg_html
            and '/login.js' in lg_html
            and "script-src 'self'" in csp_val
            and "'unsafe-inline'" not in csp_val.split("style-src")[0]
            and "frame-ancestors 'none'" in csp_val
            and "no-store" in h_lg.get("cache-control", "").lower()
            and h_lg.get("x-content-type-options") == "nosniff"
            and h_lg.get("x-frame-options") == "DENY"
            and "same-origin" in h_lg.get("cross-origin-opener-policy", ""),
            "High",
            f"xfo={h_lg.get('x-frame-options')}, nosniff={h_lg.get('x-content-type-options')}",
        )

        # Full Keycloak OIDC PKCE S256 roundtrip (allowed user + state tamper + disallowed user)
        st_ol, h_ol, sc_ol, _ = raw_http(app_port, "GET", "/api/auth/oidc/login?next=/keys")
        tx_cookie = extract_cookie(sc_ol, "lnc_styles_oidc_tx")
        auth_loc = h_ol.get("location", "")
        qp = dict(urllib.parse.parse_qsl(urllib.parse.urlsplit(auth_loc).query))

        # Tampered state must fail
        st_bad_st, h_bad_st, sc_bad_st, _ = raw_http(
            app_port,
            "GET",
            "/api/auth/oidc/callback?code=any&state=tampered_state",
            {"Cookie": tx_cookie or ""},
        )

        # Valid OIDC login for allowed user `louis`
        st_ol2, h_ol2, sc_ol2, _ = raw_http(app_port, "GET", "/api/auth/oidc/login?next=/keys")
        tx_cookie2 = extract_cookie(sc_ol2, "lnc_styles_oidc_tx") or ""
        qp2 = dict(urllib.parse.parse_qsl(urllib.parse.urlsplit(h_ol2.get("location", "")).query))
        idp.codes["code-louis"] = {
            "code_challenge": qp2["code_challenge"],
            "nonce": qp2["nonce"],
            "username": "louis",
        }
        st_cb, h_cb, sc_cb, _ = raw_http(
            app_port,
            "GET",
            f"/api/auth/oidc/callback?code=code-louis&state={urllib.parse.quote(qp2['state'])}",
            {"Cookie": tx_cookie2},
        )
        oidc_sess = extract_cookie(sc_cb, "lnc_styles_session") or ""
        st_sess, _, _, b_sess = raw_http(
            app_port, "GET", "/api/auth/session", {"Cookie": oidc_sess}
        )
        sess_info = json.loads(b_sess)

        # Disallowed OIDC user `intruder` must be denied
        st_ol3, h_ol3, sc_ol3, _ = raw_http(app_port, "GET", "/api/auth/oidc/login?next=/keys")
        tx_cookie3 = extract_cookie(sc_ol3, "lnc_styles_oidc_tx") or ""
        qp3 = dict(urllib.parse.parse_qsl(urllib.parse.urlsplit(h_ol3.get("location", "")).query))
        idp.codes["code-intruder"] = {
            "code_challenge": qp3["code_challenge"],
            "nonce": qp3["nonce"],
            "username": "intruder",
        }
        st_cb_deny, h_cb_deny, sc_cb_deny, _ = raw_http(
            app_port,
            "GET",
            f"/api/auth/oidc/callback?code=code-intruder&state={urllib.parse.quote(qp3['state'])}",
            {"Cookie": tx_cookie3},
        )
        deny_sess = extract_cookie(sc_cb_deny, "lnc_styles_session")

        runner.record(
            "security",
            "S05",
            "OIDC PKCE S256 + RS256 JWKS + nonce + allowlist + state tamper rejection",
            st_ol == 302
            and qp.get("code_challenge_method") == "S256"
            and "error=state_mismatch" in h_bad_st.get("location", "")
            and extract_cookie(sc_bad_st, "lnc_styles_session") is None
            and st_cb == 302
            and h_cb.get("location") == "/keys"
            and oidc_sess.startswith("lnc_styles_session=v2.")
            and st_sess == 200
            and sess_info.get("authenticated") is True
            and sess_info.get("mode") == "oidc"
            and "error=not_allowed" in h_cb_deny.get("location", "")
            and deny_sess is None,
            "Critical",
            f"oidc_user={sess_info.get('sub')}, deny_loc={h_cb_deny.get('location')}",
        )

        # Zero-F12-Leak & Vault Copy key lifecycle: create -> GET masked -> POST /copy 200 -> GET /copy 405 -> rotate -> revoke -> delete
        st_kc, _, _, b_kc = raw_http(
            app_port,
            "POST",
            "/api/keys",
            {"Cookie": oidc_sess, "Content-Type": "application/json"},
            json.dumps({"label": "E2E Agent Key"}),
        )
        kc_data = json.loads(b_kc)
        created_raw = kc_data.get("key", "")
        created_id = kc_data.get("item", {}).get("id", "")

        st_kl, _, _, b_kl = raw_http(app_port, "GET", "/api/keys", {"Cookie": oidc_sess})
        kl_text = b_kl.decode("utf-8")
        legacy_ids = [
            k.get("id", "")
            for k in json.loads(kl_text).get("keys", [])
            if k.get("id") != created_id and not k.get("copy_available", True)
        ]
        legacy_copy_ok = True
        if legacy_ids:
            st_legacy_copy, _, _, _ = raw_http(
                app_port,
                "POST",
                f"/api/keys/{legacy_ids[0]}/copy",
                {"Cookie": oidc_sess, "X-Forwarded-For": "10.88.0.1"},
            )
            legacy_copy_ok = st_legacy_copy == 410

        st_copy_get, _, _, _ = raw_http(
            app_port, "GET", f"/api/keys/{created_id}/copy", {"Cookie": oidc_sess}
        )
        st_copy, h_copy, _, b_copy = raw_http(
            app_port,
            "POST",
            f"/api/keys/{created_id}/copy",
            {"Cookie": oidc_sess, "X-Forwarded-For": "10.88.0.2"},
        )
        copy_json = json.loads(b_copy) if st_copy == 200 else {}
        st_copy_admin, _, _, b_copy_admin = raw_http(
            app_port,
            "POST",
            f"/api/admin/api-keys/{created_id}/copy",
            {"Cookie": oidc_sess, "X-Forwarded-For": "10.88.0.2"},
        )
        copy_admin_json = json.loads(b_copy_admin) if st_copy_admin == 200 else {}

        # Copy 15/5m/IP rate limit check on dedicated IP
        copy_429 = False
        for _ in range(16):
            st_lim, h_lim, _, _ = raw_http(
                app_port,
                "POST",
                f"/api/keys/{created_id}/copy",
                {"Cookie": oidc_sess, "X-Forwarded-For": "10.88.0.99"},
            )
            if st_lim == 429 and "retry-after" in h_lim:
                copy_429 = True
                break

        st_rot, _, _, b_rot = raw_http(
            app_port,
            "POST",
            f"/api/keys/{created_id}/rotate",
            {"Cookie": oidc_sess, "Content-Type": "application/json"},
            "{}",
        )
        rotated_raw = json.loads(b_rot).get("key", "")

        # Old token rejected on /mcp, rotated token accepted
        st_old_tok, _, _, _ = raw_http(
            app_port,
            "POST",
            "/mcp",
            {
                "Authorization": f"Bearer {created_raw}",
                "Content-Type": "application/json",
            },
            json.dumps({"jsonrpc": "2.0", "id": 1, "method": "tools/list"}),
        )
        st_new_tok, _, _, _ = raw_http(
            app_port,
            "POST",
            "/mcp",
            {
                "Authorization": f"Bearer {rotated_raw}",
                "Content-Type": "application/json",
            },
            json.dumps({"jsonrpc": "2.0", "id": 1, "method": "tools/list"}),
        )

        # Revoke & delete
        st_revk, _, _, _ = raw_http(
            app_port,
            "POST",
            f"/api/keys/{created_id}/revoke",
            {"Cookie": oidc_sess, "Content-Type": "application/json"},
            json.dumps({"revoke": True}),
        )
        st_rev_tok, _, _, _ = raw_http(
            app_port,
            "POST",
            "/mcp",
            {
                "Authorization": f"Bearer {rotated_raw}",
                "Content-Type": "application/json",
            },
            json.dumps({"jsonrpc": "2.0", "id": 1, "method": "tools/list"}),
        )
        st_delk, _, _, _ = raw_http(
            app_port, "DELETE", f"/api/keys/{created_id}", {"Cookie": oidc_sess}
        )

        # Inbound MCP Connector CRUD + Copy + Test + Rotate + Revoke + Delete
        st_conn_c, _, _, b_conn_c = raw_http(
            app_port,
            "POST",
            "/api/admin/mcp-connectors",
            {"Cookie": oidc_sess, "Content-Type": "application/json"},
            json.dumps({"name": "E2E MCP Connector", "scopes": ["mcp:read"]}),
        )
        conn_id = json.loads(b_conn_c).get("item", {}).get("id", "") if st_conn_c in (200, 201) else ""
        st_conn_copy, _, _, b_conn_copy = raw_http(
            app_port,
            "POST",
            f"/api/admin/mcp-connectors/{conn_id}/copy",
            {"Cookie": oidc_sess, "X-Forwarded-For": "10.88.0.3"},
        )
        st_conn_test, _, _, b_conn_test = raw_http(
            app_port,
            "POST",
            f"/api/admin/mcp-connectors/{conn_id}/test",
            {"Cookie": oidc_sess},
        )
        st_conn_del, _, _, _ = raw_http(
            app_port,
            "DELETE",
            f"/api/admin/mcp-connectors/{conn_id}",
            {"Cookie": oidc_sess},
        )

        runner.record(
            "security",
            "S06",
            "Zero-F12-Leak vault key & MCP connector lifecycle (masked GET, HKDF POST /copy + 15/5m 429, 405 GET /copy, legacy 410, rotate/revoke/delete)",
            st_kc in (200, 201)
            and created_raw.startswith("styles_")
            and st_kl == 200
            and "raw_key" not in kl_text
            and created_raw not in kl_text
            and LEGACY_PLAINTEXT_TOKEN not in kl_text
            and st_copy_get == 405
            and st_copy == 200
            and "no-store" in h_copy.get("cache-control", "").lower()
            and copy_json.get("secret") == created_raw
            and st_copy_admin == 200
            and copy_admin_json.get("secret") == created_raw
            and copy_429
            and legacy_copy_ok
            and st_rot == 200
            and rotated_raw.startswith("styles_")
            and rotated_raw != created_raw
            and st_old_tok == 401
            and st_new_tok == 200
            and st_revk == 200
            and st_rev_tok == 401
            and st_delk == 200
            and st_conn_c in (200, 201)
            and st_conn_copy == 200
            and json.loads(b_conn_copy).get("secret") == json.loads(b_conn_c).get("secret")
            and st_conn_test == 200
            and json.loads(b_conn_test).get("ok") is True
            and st_conn_del == 200,
            "Critical",
            f"copy=({st_copy},{st_copy_admin},429={copy_429}), legacy410={legacy_copy_ok}, conn=({st_conn_c},{st_conn_copy},{st_conn_test},{st_conn_del})",
        )

        # CSRF protection, logout Origin+X-CSRF-Token strict guard, ?token= 400 block, SSRF metadata block & session_epoch revoke-all
        _, _, _, b_sess_csrf = raw_http(
            app_port, "GET", "/api/auth/session", {"Cookie": oidc_sess}
        )
        csrf_tok = json.loads(b_sess_csrf).get("csrf_token") or ""
        st_logout_get, _, _, _ = raw_http(
            app_port, "GET", "/api/auth/logout", {"Cookie": oidc_sess}
        )
        st_logout_spoof, _, _, _ = raw_http(
            app_port,
            "POST",
            "/api/auth/logout",
            {
                "Cookie": oidc_sess,
                "Origin": "https://spoofed.evil.example",
                "X-Forwarded-Host": "spoofed.evil.example",
                "X-CSRF-Token": csrf_tok,
            },
        )
        st_logout_nocsrf, _, _, _ = raw_http(
            app_port,
            "POST",
            "/api/auth/logout",
            {
                "Cookie": oidc_sess,
                "Origin": f"http://127.0.0.1:{app_port}",
            },
        )
        st_logout_ok, _, sc_logout_ok, _ = raw_http(
            app_port,
            "POST",
            "/api/auth/logout",
            {
                "Cookie": oidc_sess,
                "Origin": f"http://127.0.0.1:{app_port}",
                "X-CSRF-Token": csrf_tok,
            },
        )
        st_q_tok, _, _, _ = raw_http(app_port, "GET", "/api/styles?token=secret_leak")
        st_q_key, _, _, _ = raw_http(app_port, "GET", "/api/styles?key=secret_leak")
        st_q_apik, _, _, _ = raw_http(app_port, "GET", "/mcp?api_key=secret_leak")

        st_csrf, _, _, _ = raw_http(
            app_port,
            "POST",
            "/api/keys",
            {
                "Cookie": oidc_sess,
                "Origin": "https://attacker.evil.example",
                "Content-Type": "application/json",
            },
            json.dumps({"label": "CSRF Key"}),
        )
        st_ssrf1, _, _, _ = raw_http(
            app_port,
            "PUT",
            "/api/admin/settings",
            {"Cookie": oidc_sess, "Content-Type": "application/json"},
            json.dumps({"public_base_url": "http://169.254.169.254/latest/meta-data/"}),
        )
        st_ssrf2, _, _, _ = raw_http(
            app_port,
            "PUT",
            "/api/admin/settings",
            {"Cookie": oidc_sess, "Content-Type": "application/json"},
            json.dumps({"public_base_url": "http://metadata.google.internal/computeMetadata/v1/"}),
        )
        st_rev_all, _, _, b_rev_all = raw_http(
            app_port,
            "POST",
            "/api/auth/revoke-all",
            {"Cookie": oidc_sess, "X-CSRF-Token": csrf_tok},
        )
        st_after_rev, _, _, _ = raw_http(
            app_port, "GET", "/api/keys", {"Cookie": oidc_sess}
        )
        runner.record(
            "security",
            "S07",
            "CSRF Origin+Token guard (403/405), ?token= 400 block, SSRF metadata block (400) & revoke-all session_epoch invalidation",
            st_csrf == 403
            and st_logout_get == 405
            and st_logout_spoof == 403
            and st_logout_nocsrf == 403
            and st_logout_ok == 200
            and any("max-age=0" in sc.lower() for sc in sc_logout_ok)
            and st_q_tok == 400
            and st_q_key == 400
            and st_q_apik == 400
            and st_ssrf1 == 400
            and st_ssrf2 == 400
            and st_rev_all == 200
            and json.loads(b_rev_all).get("session_epoch", 0) >= 2
            and st_after_rev == 401,
            "High",
            f"csrf={st_csrf}, logout=({st_logout_get},{st_logout_spoof},{st_logout_nocsrf},{st_logout_ok}), q=({st_q_tok},{st_q_key},{st_q_apik}), ssrf=({st_ssrf1},{st_ssrf2}), post_revoke_all={st_after_rev}",
        )

        # Break-glass Argon2id login + 8/5m rate-limit -> 429 + Retry-After
        st_bg_ok, _, sc_bg, _ = raw_http(
            app_port,
            "POST",
            "/api/auth/login",
            {"Content-Type": "application/json", "X-Forwarded-For": "10.99.0.1"},
            json.dumps({"key": MGMT_KEY}),
        )
        mgmt_sess = extract_cookie(sc_bg, "lnc_styles_session") or ""

        bad_statuses: list[int] = []
        for _ in range(8):
            s_b, _, _, _ = raw_http(
                app_port,
                "POST",
                "/api/auth/login",
                {"Content-Type": "application/json", "X-Forwarded-For": "10.99.0.2"},
                json.dumps({"key": "wrong-mgmt-secret"}),
            )
            bad_statuses.append(s_b)
        st_9th, h_9th, _, _ = raw_http(
            app_port,
            "POST",
            "/api/auth/login",
            {"Content-Type": "application/json", "X-Forwarded-For": "10.99.0.2"},
            json.dumps({"key": "wrong-mgmt-secret"}),
        )
        runner.record(
            "security",
            "S08",
            "Break-glass Argon2id login succeeds & 9th failed attempt returns 429 + Retry-After",
            st_bg_ok == 200
            and mgmt_sess.startswith("lnc_styles_session=v2.")
            and all(x == 401 for x in bad_statuses)
            and st_9th == 429
            and int(h_9th.get("retry-after", "0")) > 0,
            "High",
            f"mgmt_login={st_bg_ok}, 9th_status={st_9th}, retry_after={h_9th.get('retry-after')}s",
        )

        # Path traversal probes
        trav_probes = [
            "/api/styles/..%2Findex.json",
            "/api/templates/en-01-saas-landing/..%2Fprompts-en%2F01-saas-landing.md",
            "/api/components/lnc-neon-btn/files/..%2F..%2Findex.json",
            "/..%2FCargo.toml",
            "/static/%2e%2e/VERSION",
        ]
        trav_codes = [raw_http(app_port, "GET", p)[0] for p in trav_probes]
        runner.record(
            "security",
            "S09",
            "Path traversal probes rejected across styles, templates, components & static routes",
            all(c in {400, 404} for c in trav_codes),
            "High",
            f"probe_statuses={trav_codes}",
        )

        # ------------------------------------------------------------------
        # Category 4: MCP Streamable HTTP JSON-RPC 2.0 Parity (20%)
        # ------------------------------------------------------------------
        mcp_hdrs = {
            "Authorization": f"Bearer {LEGACY_PLAINTEXT_TOKEN}",
            "Content-Type": "application/json",
        }

        def mcp_call(payload: dict[str, Any]) -> tuple[int, Any]:
            s, _, _, raw_b = raw_http(
                app_port, "POST", "/mcp", mcp_hdrs, json.dumps(payload)
            )
            return s, (json.loads(raw_b) if raw_b else None)

        s_init, j_init = mcp_call(
            {"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}}
        )
        s_notif, j_notif = mcp_call(
            {"jsonrpc": "2.0", "method": "notifications/initialized"}
        )
        s_list, j_list = mcp_call({"jsonrpc": "2.0", "id": 2, "method": "tools/list"})
        tool_names = [t["name"] for t in (j_list or {}).get("result", {}).get("tools", [])]
        expected_tools = {
            "list_styles",
            "search",
            "get_design",
            "list_collections",
            "get_template",
            "list_components",
            "get_component",
            "reindex",
        }
        runner.record(
            "mcp",
            "M01",
            "MCP `initialize`, `notifications/initialized` (HTTP 202) & `tools/list` (all 8 tools)",
            s_init == 200
            and j_init["result"]["serverInfo"]["version"] == "1.0.0.1"
            and s_notif == 202
            and j_notif is None
            and s_list == 200
            and set(tool_names) == expected_tools,
            "Critical",
            f"notif={s_notif}, tools={len(tool_names)} ({','.join(tool_names)})",
        )

        # Invoke all 8 MCP tools (mixing direct names and Hub `styles__*` prefixed names)
        calls = [
            ("list_styles", {"theme": "dark", "limit": 5}),
            ("styles__search", {"query": "copilot", "limit": 5}),
            ("styles__get_design", {"style_id": "s-obsidian-cli"}),
            ("list_collections", {}),
            ("get_template", {"template_id": "en-01-saas-landing", "include_html": True}),
            ("list_components", {"kind": "tsx"}),
            ("styles__get_component", {"component_id": "knowns-agent-plan"}),
            ("styles__reindex", {}),
        ]
        tool_ok_count = 0
        for idx, (tname, targs) in enumerate(calls, start=10):
            sc, jc = mcp_call(
                {
                    "jsonrpc": "2.0",
                    "id": idx,
                    "method": "tools/call",
                    "params": {"name": tname, "arguments": targs},
                }
            )
            res_obj = (jc or {}).get("result", {})
            if sc == 200 and not res_obj.get("isError") and res_obj.get("content"):
                tool_ok_count += 1

        runner.record(
            "mcp",
            "M02",
            "All 8 MCP tools execute cleanly (both unprefixed and `styles__*` prefixed)",
            tool_ok_count == 8,
            "Critical",
            f"tools_passed={tool_ok_count}/8",
        )

        # Unknown method & unknown tool error semantics
        s_unk_m, j_unk_m = mcp_call({"jsonrpc": "2.0", "id": 99, "method": "foo/bar"})
        s_unk_t, j_unk_t = mcp_call(
            {
                "jsonrpc": "2.0",
                "id": 100,
                "method": "tools/call",
                "params": {"name": "nonexistent_tool", "arguments": {}},
            }
        )
        runner.record(
            "mcp",
            "M03",
            "JSON-RPC `-32601` on unknown method & `isError: true` on unknown tool",
            s_unk_m == 200
            and j_unk_m.get("error", {}).get("code") == -32601
            and s_unk_t == 200
            and j_unk_t.get("result", {}).get("isError") is True,
            "Medium",
            f"method_err={j_unk_m.get('error', {}).get('code')}, tool_isError={j_unk_t.get('result', {}).get('isError')}",
        )

        # ------------------------------------------------------------------
        # Category 5: Failure Recovery, Resilience & Corpus Immutability (15%)
        # ------------------------------------------------------------------
        st_bad_json, _, _, _ = raw_http(
            app_port,
            "POST",
            "/mcp",
            mcp_hdrs,
            "{malformed-json",
        )
        st_miss_style, _, _, _ = raw_http(app_port, "GET", "/api/styles/does-not-exist")
        st_miss_tpl, _, _, _ = raw_http(app_port, "GET", "/api/templates/does-not-exist")
        st_miss_cmp, _, _, _ = raw_http(app_port, "GET", "/api/components/does-not-exist")
        st_reidx, _, _, b_reidx = raw_http(
            app_port, "POST", "/api/admin/reindex", {"Cookie": mgmt_sess}
        )
        runner.record(
            "resilience",
            "R01",
            "Graceful error handling (malformed JSON, missing entities) & hot `/api/admin/reindex`",
            st_bad_json == 400
            and st_miss_style == 404
            and st_miss_tpl == 404
            and st_miss_cmp == 404
            and st_reidx == 200
            and json.loads(b_reidx).get("status") == "reindexed",
            "Medium",
            f"bad_json={st_bad_json}, 404s=({st_miss_style},{st_miss_tpl},{st_miss_cmp}), reindex={st_reidx}",
        )

        corpus_after = snapshot_tree(data_dir)
        runner.record(
            "resilience",
            "R02",
            "Read-only `${STYLES_DATA_DIR}` corpus completely unmutated across boot, queries & reindex",
            corpus_before == corpus_after,
            "High",
            f"files_tracked={len(corpus_before)}, mutated={corpus_before != corpus_after}",
        )

    finally:
        if proc is not None and proc.poll() is None:
            proc.terminate()
            try:
                proc.wait(timeout=5)
            except subprocess.TimeoutExpired:
                proc.kill()
        idp.stop()
        shutil.rmtree(tmp_root, ignore_errors=True)

    # Compute weighted Track M score:
    # journeys=25, security=30, mcp=20, resilience=15, health=10
    weights = {
        "journeys": 25.0,
        "security": 30.0,
        "mcp": 20.0,
        "resilience": 15.0,
        "health": 10.0,
    }
    cat_scores: dict[str, tuple[float, float, int, int]] = {}
    total_score = 0.0
    for cat, w in weights.items():
        items = [c for c in runner.checks if c["category"] == cat]
        passed_n = sum(1 for c in items if c["passed"])
        total_n = len(items)
        earned = round(w * (passed_n / total_n), 2) if total_n else 0.0
        cat_scores[cat] = (earned, w, passed_n, total_n)
        total_score += earned
    total_score = round(total_score, 2)

    crit_fails = sum(
        1 for c in runner.checks if not c["passed"] and c["severity"] == "Critical"
    )
    high_fails = sum(
        1 for c in runner.checks if not c["passed"] and c["severity"] == "High"
    )
    med_fails = sum(
        1 for c in runner.checks if not c["passed"] and c["severity"] == "Medium"
    )
    low_fails = sum(
        1 for c in runner.checks if not c["passed"] and c["severity"] == "Low"
    )
    hard_fail = "TRIGGERED" if (crit_fails > 0 or high_fails > 0) else "CLEAR"
    verdict = (
        "RELEASE_READY (SHIP / CUTOVER)"
        if total_score >= 90.0 and hard_fail == "CLEAR"
        else "FIX_REQUIRED"
    )

    EVIDENCE_DIR.mkdir(parents=True, exist_ok=True)
    baseline_payload = {
        "service": "lnc-styles-gallery",
        "version": "1.0.0.1",
        "track": "Track M (Migration & Security Hardening)",
        "generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "total_score": total_score,
        "verdict": verdict,
        "hard_fail": hard_fail,
        "failures": {
            "critical": crit_fails,
            "high": high_fails,
            "medium": med_fails,
            "low": low_fails,
        },
        "metrics": runner.metrics,
        "checks": runner.checks,
    }
    (EVIDENCE_DIR / "baseline.json").write_text(
        json.dumps(baseline_payload, indent=2), encoding="utf-8"
    )

    cat_labels = {
        "journeys": "Core User Journeys & Corpus REST Parity",
        "security": "Security, Crypto, OIDC & Zero-F12-Leak Parity",
        "mcp": "MCP Streamable HTTP JSON-RPC 2.0 Parity",
        "resilience": "Failure Recovery & Read-Only Corpus Resilience",
        "health": "Build, Startup, Latency & Footprint",
    }
    md_lines = [
        "# E2E & Migration Parity Scorecard — `lnc-styles-gallery` (`v1.0.0.1`)",
        "",
        f"- **Track**: Track M (Python/FastAPI -> Rust/Axum Migration + Security Hardening)",
        f"- **Final Weighted Score**: **{total_score:.1f} / 100**",
        f"- **Hard-Fail Gate**: **{hard_fail}** (`Critical={crit_fails}`, `High={high_fails}`, `Medium={med_fails}`, `Low={low_fails}`)",
        f"- **Release Decision**: **{verdict}**",
        f"- **Runtime Metrics**: Binary `{runner.metrics.get('binary_size_bytes', 0):,}` bytes | Boot `{runner.metrics.get('boot_ms')} ms` | Idle RSS `{runner.metrics.get('idle_rss_mb')} MB` | Post-Load RSS `{runner.metrics.get('post_load_rss_mb')} MB` | Latency `p50={runner.metrics.get('p50_ms')} ms`, `p95={runner.metrics.get('p95_ms')} ms`",
        "",
        "## Weighted Category Breakdown",
        "",
        "| Category | Weight | Checks Passed | Score |",
        "| :--- | :---: | :---: | :---: |",
    ]
    for cat in ["journeys", "security", "mcp", "resilience", "health"]:
        earned, w, pn, tn = cat_scores[cat]
        md_lines.append(
            f"| {cat_labels[cat]} | {int(w)}% | `{pn}/{tn}` | **{earned:.1f} / {int(w)}** |"
        )
    md_lines.extend(
        [
            "",
            "## Detailed Check Results",
            "",
            "| ID | Category | Severity | Status | Check | Evidence |",
            "| :--- | :--- | :---: | :---: | :--- | :--- |",
        ]
    )
    for c in runner.checks:
        st_str = "PASS" if c["passed"] else "FAIL"
        md_lines.append(
            f"| `{c['id']}` | `{c['category']}` | {c['severity']} | **{st_str}** | {c['title']} | {c['detail']} |"
        )
    md_lines.append("")
    (EVIDENCE_DIR / "scorecard.md").write_text("\n".join(md_lines), encoding="utf-8")

    print(
        f"\nSCORE: {total_score:.1f}/100 | Hard-Fail: {hard_fail} | Verdict: {verdict}"
    )
    print(f"Evidence written to: {EVIDENCE_DIR / 'scorecard.md'}")
    return 0 if (total_score >= 90.0 and hard_fail == "CLEAR") else 1


if __name__ == "__main__":
    sys.exit(run_scorecard())
