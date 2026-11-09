import os
from pathlib import Path
from typing import Dict, Any

ROOT_DIR = Path(__file__).resolve().parent.parent
VERSION_FILE = ROOT_DIR / "VERSION"

NAS_REFERO_DEFAULT = Path(r"\\10.21.2.2\data\_agentsync\repos\refero-styles")
NAS_PROMPT_UI_DEFAULT = Path(r"\\10.21.2.2\data\_agentsync\repos\prompt-driven-ui")
NAS_COMPONENTS_DEFAULT = Path(r"\\10.21.2.2\data\_agentsync\repos\lnc-components")


def get_version() -> str:
    try:
        if VERSION_FILE.exists():
            return VERSION_FILE.read_text(encoding="utf-8").strip() or "0.1.0"
    except Exception:
        pass
    return "0.1.0"


class Settings:
    def __init__(self) -> None:
        self.service_name: str = "lnc-styles-gallery"
        self.app_title: str = "LNC Style Library"
        self.version: str = get_version()
        self.port: int = int(os.environ.get("PORT", "8460"))
        self.public_url: str = os.environ.get("STYLES_PUBLIC_URL", "https://styles.lengoc.me")
        self.api_key: str = os.environ.get("STYLES_API_KEY", os.environ.get("ADMIN_API_KEY", "")).strip()
        self.data_dir: Path = Path(os.environ.get("STYLES_DATA_DIR", str(ROOT_DIR / "data")))
        self._refero_override: str = os.environ.get("REFERO_STYLES_DIR", "").strip()
        self._prompt_ui_override: str = os.environ.get("PROMPT_UI_DIR", "").strip()
        self._components_override: str = os.environ.get("COMPONENTS_DIR", "").strip()
        self.auto_warm_cache: bool = os.environ.get("STYLES_AUTO_WARM_CACHE", "false").lower() in ("1", "true", "yes")

    def resolve_refero_dir(self) -> Path:
        if self._refero_override:
            return Path(self._refero_override)
        local_candidate = self.data_dir / "refero-styles"
        try:
            if (local_candidate / "index.json").exists() or (local_candidate / "styles").exists():
                return local_candidate
        except Exception:
            pass
        try:
            if NAS_REFERO_DEFAULT.exists():
                return NAS_REFERO_DEFAULT
        except Exception:
            pass
        return local_candidate

    def resolve_prompt_ui_dir(self) -> Path:
        if self._prompt_ui_override:
            return Path(self._prompt_ui_override)
        local_candidate = self.data_dir / "prompt-driven-ui"
        try:
            if (local_candidate / "websites-en").exists() or (local_candidate / "websites").exists():
                return local_candidate
        except Exception:
            pass
        try:
            if NAS_PROMPT_UI_DEFAULT.exists():
                return NAS_PROMPT_UI_DEFAULT
        except Exception:
            pass
        return local_candidate

    def resolve_components_dir(self) -> Path:
        if self._components_override:
            return Path(self._components_override)
        local_candidate = self.data_dir / "lnc-components"
        try:
            if (local_candidate / "index.json").exists() or (local_candidate / "items").exists():
                return local_candidate
        except Exception:
            pass
        try:
            if NAS_COMPONENTS_DEFAULT.exists():
                return NAS_COMPONENTS_DEFAULT
        except Exception:
            pass
        return local_candidate

    def masked_dict(self) -> Dict[str, Any]:
        refero_dir = self.resolve_refero_dir()
        prompt_ui_dir = self.resolve_prompt_ui_dir()
        components_dir = self.resolve_components_dir()
        return {
            "service_name": self.service_name,
            "app_title": self.app_title,
            "version": self.version,
            "port": self.port,
            "public_url": self.public_url,
            "data_dir": str(self.data_dir),
            "refero_styles_dir": str(refero_dir),
            "prompt_ui_dir": str(prompt_ui_dir),
            "components_dir": str(components_dir),
            "auth_mode": "api_key" if self.api_key else "internal_open",
            "api_key_configured": bool(self.api_key),
            "api_key_masked": ("***" + self.api_key[-4:]) if len(self.api_key) > 4 else ("***" if self.api_key else ""),
            "auto_warm_cache": self.auto_warm_cache,
        }


settings = Settings()
