import json
import secrets
from datetime import datetime, timezone
from pathlib import Path
from typing import List, Dict, Optional, Any

from app.config import settings


class KeyService:
    def __init__(self) -> None:
        self._keys_file: Optional[Path] = None

    @property
    def keys_file(self) -> Path:
        if self._keys_file is None:
            p = settings.config_dir / "keys.json"
            p.parent.mkdir(parents=True, exist_ok=True)
            self._keys_file = p
        return self._keys_file

    def load_data(self) -> Dict[str, Any]:
        f = self.keys_file
        if f.exists():
            try:
                with open(f, "r", encoding="utf-8") as fp:
                    return json.load(fp)
            except Exception as e:
                print(f"[KeyService] Error loading keys.json: {e}")
        return {"api_keys": []}

    def save_data(self, data: Dict[str, Any]) -> None:
        f = self.keys_file
        with open(f, "w", encoding="utf-8") as fp:
            json.dump(data, fp, indent=2)

    def ensure_seed_key(self) -> str:
        """Ensures at least one active API key exists for Business MCP Hub and returns it."""
        data = self.load_data()
        keys = data.get("api_keys", [])
        if keys:
            for k in keys:
                if k.get("enabled", True):
                    return k.get("key", "")

        seed_key = f"styles_{secrets.token_urlsafe(24)}"
        entry = {
            "id": "hub_default",
            "label": "Business MCP Hub",
            "key": seed_key,
            "enabled": True,
            "created": datetime.now(timezone.utc).isoformat(),
        }
        data["api_keys"] = [entry]
        self.save_data(data)
        print(f"[KeyService] Seeded default API key for Business MCP Hub: {seed_key[:12]}...")
        return seed_key

    def list_keys(self) -> List[Dict[str, Any]]:
        self.ensure_seed_key()
        data = self.load_data()
        results = []
        for k in data.get("api_keys", []):
            raw_key = k.get("key", "")
            prefix = raw_key[:12] + "..." if len(raw_key) > 12 else raw_key
            results.append({
                "id": k.get("id"),
                "label": k.get("label"),
                "prefix": prefix,
                "raw_key": raw_key,
                "enabled": k.get("enabled", True),
                "created": k.get("created", ""),
                "rotated_at": k.get("rotated_at", ""),
            })
        return results

    def create_key(self, label: str, custom_secret: Optional[str] = None) -> Dict[str, Any]:
        data = self.load_data()
        token = custom_secret.strip() if custom_secret and custom_secret.strip() else f"styles_{secrets.token_urlsafe(24)}"
        entry = {
            "id": secrets.token_hex(6),
            "label": label or "Unnamed API Key",
            "key": token,
            "enabled": True,
            "created": datetime.now(timezone.utc).isoformat(),
        }
        data.setdefault("api_keys", []).append(entry)
        self.save_data(data)
        return entry

    def revoke_key(self, key_id: str, revoke: bool = True) -> Optional[Dict[str, Any]]:
        """Toggles active/revoked status of an API key without deleting it."""
        data = self.load_data()
        for k in data.get("api_keys", []):
            if k.get("id") == key_id:
                k["enabled"] = not revoke
                k["revoked_at"] = datetime.now(timezone.utc).isoformat() if revoke else None
                self.save_data(data)
                return k
        return None

    def rotate_key(self, key_id: str, custom_secret: Optional[str] = None) -> Optional[Dict[str, Any]]:
        """Generates a new token for an existing key entry."""
        data = self.load_data()
        for k in data.get("api_keys", []):
            if k.get("id") == key_id:
                new_token = custom_secret.strip() if custom_secret and custom_secret.strip() else f"styles_{secrets.token_urlsafe(24)}"
                k["key"] = new_token
                k["enabled"] = True
                k["rotated_at"] = datetime.now(timezone.utc).isoformat()
                self.save_data(data)
                return {
                    "id": k["id"],
                    "label": k.get("label", ""),
                    "key": new_token,
                    "enabled": True,
                    "rotated_at": k["rotated_at"],
                }
        return None

    def delete_key(self, key_id: str) -> bool:
        """Permanently deletes an API key."""
        data = self.load_data()
        keys = data.get("api_keys", [])
        initial_len = len(keys)
        data["api_keys"] = [k for k in keys if k.get("id") != key_id]
        if len(data["api_keys"]) < initial_len:
            self.save_data(data)
            return True
        return False

    def validate_key(self, token: str) -> bool:
        if not token:
            return False
        if settings.api_key and token == settings.api_key:
            return True
        data = self.load_data()
        for k in data.get("api_keys", []):
            if k.get("enabled", True) and k.get("key") == token:
                return True
        return False


key_service = KeyService()
