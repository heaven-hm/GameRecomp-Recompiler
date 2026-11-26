"""Guard the pinned GameReComp provenance documented in the SDK."""
from pathlib import Path
import re
ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "docs/UPSTREAM_SYNC.md"

def test_pinned_upstream_refs():
    text = DOC.read_text()
    assert "cc35fef790ab52c881d31746a29e55f22b3397cb" in text
    assert "0be18e9981c53c50d9cd315f92e2cfd658d10063" in text
    assert "0be18e99" in text and "34d82e7c" in text and "c316395c" in text
    assert "releases/tag/v0.3.0" in text
    assert not re.search(r"(?i)\b(?:close[sd]?|fix(?:e[sd])?)\s+(?:issue|#)\s*\d+", text)

def test_no_vendored_recompiler():
    assert not (ROOT / "tools/autorecomp").exists()
    assert not (ROOT / "runtime/win32hle").exists()
