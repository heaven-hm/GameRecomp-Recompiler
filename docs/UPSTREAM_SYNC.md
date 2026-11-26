# GameReComp upstream code used by this SDK

This SDK is a thin client. All recompilation code lives in the separate
[GameReComp](https://github.com/heaven-hm/GameReComp) repository. This document
pins the exact upstream code referenced by the apps and proof in this repo.

| Upstream ref | SHA-256 / commit | What it provides |
|---|---|---|
| Release tag `v0.3.0` | `cc35fef790ab52c881d31746a29e55f22b3397cb` | `game-recomp-cli` v0.3.0 used by every SDK app; six release binaries + `SHA256SUMS.txt` |
| `develop` at sync time | `0be18e9981c53c50d9cd315f92e2cfd658d10063` | latest upstream code after v0.3.0 |

New upstream code after v0.3.0 (verified from the commit list):

- `0be18e99` — `fix(gui,cli): allow macOS/Linux/x64 input selection (#323)`.
  The CLI can now declare input OS/arch explicitly; this SDK keeps the stable
  default contract and does not hard-code those flags.
- `34d82e7c` — `docs(matrix): add missing priority titles with hash-verified frames (#322)`.
  New game evidence lives in GameReComp `docs/GAME_MATRIX.md` and linked frame
  files; the SDK proof pages link the game evidence section.
- `c316395c` — `evidence: Blaze Rust build attempt and stall (#321)`.
  Records the local Blaze daemon limitation; this SDK's Rust binding is verified
  with Blaze on the smaller crate.

Upstream evidence links:

- [GameReComp v0.3.0 release](https://github.com/heaven-hm/GameReComp/releases/tag/v0.3.0)
- [Issue #28 proof (real Chocolate Doom frames)](https://github.com/heaven-hm/GameReComp/blob/v0.3.0/docs/ISSUE_28_PROOF.md)
- [Game matrix with hash-verified frames](https://github.com/heaven-hm/GameReComp/blob/0be18e9981c53c50d9cd315f92e2cfd658d10063/docs/GAME_MATRIX.md)
- [Windows oracle proof](https://github.com/heaven-hm/GameReComp/tree/v0.3.0/proof/windows-oracle)

No recompiler source is vendored here: copying the lifter/runtime would create a
second recompiler and break the SDK boundary. Verify the upstream commit with:

```sh
git ls-remote https://github.com/heaven-hm/GameReComp.git refs/heads/develop
```
