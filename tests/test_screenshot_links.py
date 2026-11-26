"""Every proof screenshot link must resolve; fake/duplicated captures are gone."""
import hashlib
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DOCS = [ROOT / "README.md", ROOT / "PROOF.md", ROOT / "demos/README.md"]
DOCS += sorted((ROOT / "docs").glob("*.md"))
LINK = re.compile(r"!?\[[^]]*\]\(([^)]+)\)")

def test_all_screenshot_links_exist():
    checked = 0
    for doc in DOCS:
        text = doc.read_text()
        for target in LINK.findall(text):
            if "screenshots/" not in target or target.endswith("/"):
                continue
            path = (doc.parent / target).resolve()
            assert path.is_file(), f"broken screenshot link in {doc.name}: {target}"
            checked += 1
    assert checked >= 10, f"expected screenshot links in docs, found {checked}"

def test_only_real_distinct_terminal_captures():
    files = sorted((ROOT / "proof/screenshots").iterdir())
    names = [f.name for f in files]
    assert names == [f"terminal-{s}.png" for s in
                     ("c", "cpp", "javascript", "python", "rust")], names
    digests = [hashlib.sha256(f.read_bytes()).hexdigest() for f in files]
    assert len(set(digests)) == 5, "screenshots are not distinct"
