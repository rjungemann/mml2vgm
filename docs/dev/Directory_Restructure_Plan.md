# Directory Restructure Plan

The project has grown organically and the layout no longer reflects the logical boundaries between its components. This document proposes a target structure and a migration sequence.

---

## Current Problems

| Area | Problem |
|------|---------|
| `docs/` | ~55 files mixing user docs, design specs, dev ops guides, PHASE2 progress tracking, binary blobs (PDFs, ZIPs, HDI), ROM dumps, and instrument packs |
| Root level | Stray files: `VALIDATION_FRAMEWORK_STATUS.md`, `COMPILER_FIX_SUMMARY.md`, `smoke-test.mjs`, `test_sf2_golden_long.wav` |
| `mml2vgmTest/` | Legacy test directory with Japanese-named folders, superseded by `tests/` |
| `tools/` vs `scripts/` | Overlapping purpose — utility scripts scattered across both |
| `tools/validation/` | Validation scripts are test infrastructure, not tools |
| `validation_results/` | Generated output committed to the repo |
| `.playwright-mcp/` | Tool log artifacts committed to the repo |
| Rust crates | Three crates (`mml2vgm-rs/`, `mml2vgm-wasm/`, `egui-app/`) sit at root alongside the frontend app |

---

## Proposed Target Structure

```
mml2vgm/
├── .github/
├── apps/
│   └── browser-ide/          ← was browser-ide/
├── crates/
│   ├── mml2vgm-rs/           ← core library + CLI (unchanged internals)
│   ├── mml2vgm-wasm/         ← WASM bindings (unchanged internals)
│   └── egui-app/             ← desktop GUI (unchanged internals)
├── cfg/                      ← unchanged
├── docs/
│   ├── user/
│   │   ├── tutorial/         ← was docs/tutorial/
│   │   ├── tutorial-examples/← was docs/tutorial-examples/
│   │   ├── MML_Commands.md
│   │   ├── User_Manual.md
│   │   └── Furnace_Instruments.mml
│   ├── design/
│   │   ├── Console_Chips_Design.md
│   │   ├── Example_Tracks_Design.md
│   │   ├── External_Driver_Support.md
│   │   ├── MIDI_Export_Design.md
│   │   ├── Performance_Improvement_Design.md
│   │   ├── QSound_Design.md
│   │   ├── Sample_Format_Expansion_Design.md
│   │   ├── ZGM_Specification.md
│   │   ├── ZGMspec.txt
│   │   └── PSG2.txt
│   ├── dev/
│   │   ├── Development.md
│   │   ├── EMULATOR_SETUP.md
│   │   ├── PC98_EMULATOR_SETUP.md
│   │   ├── Cloudflare_Pages_Deployment.md
│   │   ├── Browser_IDE_Implementation.md
│   │   ├── Browser_IDE_Limitations.md
│   │   ├── LINUX_CLI_COMPLETION.md
│   │   ├── PERFORMANCE_FIXES.md
│   │   ├── Found_ROMs_Status.md
│   │   ├── ROM_ACQUISITION_GUIDE.md
│   │   ├── Rust_CLI_Design.md
│   │   └── Validation_Status.md
│   ├── archive/              ← historical / completed-phase docs
│   │   ├── reports/          ← was docs/reports/
│   │   ├── PHASE2_*.md       ← all PHASE2 progress docs
│   │   ├── Golden_Master_*.md
│   │   ├── GOLDEN_MASTER_TEST_PLAN.md
│   │   └── CHANGELOG.md
│   └── reference/            ← external reference material
│       ├── m98コマンド・リファレンス.pdf
│       ├── mml2vgm_MMLCommandMemo.txt
│       └── YM2609.txt
├── examples/                 ← unchanged (example .gwi files)
├── resources/                ← binary/data assets (gitignored or LFS)
│   ├── furnace-instruments/  ← was docs/Furnace Instruments/
│   ├── furnace-tracks/       ← was docs/Furnace Tracks/ + docs/Furnace Tracks MML/
│   ├── opl3-patch-pack/      ← was docs/OPL3 Patch Pack/
│   ├── roms/
│   │   ├── enduror/          ← was docs/enduror/
│   │   └── terracren/        ← was docs/terracren/
│   ├── bios/                 ← was docs/PC-98 BIOS Files/
│   └── sf2/                  ← was docs/sf2.zip (unpack or keep zipped)
├── scripts/                  ← all utility scripts consolidated
│   ├── smoke-test.mjs        ← was root smoke-test.mjs
│   ├── compare_vgm.mjs       ← was tools/compare_vgm.mjs
│   ├── compare_wav.mjs       ← was tools/compare_wav.mjs
│   ├── convert_demos_to_mml.py ← was tools/convert_demos_to_mml.py
│   ├── convert_fur_to_mml.py ← was tools/convert_fur_to_mml.py
│   ├── convert_sitaraba.mjs  ← was tools/convert_sitaraba.mjs
│   └── fui2mml.py            ← was tools/fui2mml.py
├── tests/                    ← all tests (keep current structure)
│   ├── golden_master/
│   └── parity/
├── tools/                    ← external tools/emulators only
│   ├── NP2kai/               ← keep
│   └── validation/           ← was tools/validation/ (golden master scripts)
├── Justfile
├── README.md
└── LICENSE.txt
```

---

## Files to Delete

| File | Reason |
|------|--------|
| `mml2vgmTest/` | Entirely superseded by `tests/`; contains Japanese-named scratch folders and one-off test VGM/WAV files with no clear ownership |
| `test_sf2_golden_long.wav` | Stray test artifact at root |
| `docs/Brandish 2 Renewal.hdi` | ROM/disk image; move to `resources/roms/` or delete if not needed |

## Files to Gitignore

| Path | Reason |
|------|--------|
| `.playwright-mcp/` | Playwright MCP log artifacts — regenerated on every run |
| `validation_results/` | Generated output; should never be committed |
| `test_sf2_golden_long.wav` | Generated test artifact |

---

## Migration Steps

The migration should be done in phases to keep `git blame` and history intact (use `git mv`, not `cp`+`rm`).

### Phase 1 — Gitignore cleanup (no history impact)
1. Add `.playwright-mcp/`, `validation_results/`, `test_sf2_golden_long.wav` to `.gitignore`
2. Remove `validation_results/` and `.playwright-mcp/` from tracking (`git rm -r --cached`)

### Phase 2 — Root cleanup
1. `git mv VALIDATION_FRAMEWORK_STATUS.md docs/dev/`
2. `git mv COMPILER_FIX_SUMMARY.md docs/archive/`
3. `git mv smoke-test.mjs scripts/smoke-test.mjs`
4. Update `package.json` / `Justfile` references to `smoke-test.mjs`

### Phase 3 — docs/ reorganization
1. Create subdirectories: `docs/user/`, `docs/design/`, `docs/dev/`, `docs/archive/`, `docs/reference/`
2. Move files per the table above using `git mv`
3. Move `docs/enduror/`, `docs/terracren/`, `docs/PC-98 BIOS Files/`, binary assets to `resources/`

### Phase 4 — scripts/ consolidation
1. `git mv tools/compare_vgm.mjs scripts/`
2. `git mv tools/compare_wav.mjs scripts/`
3. `git mv tools/convert_*.py scripts/`
4. `git mv tools/convert_sitaraba.mjs scripts/`
5. `git mv tools/fui2mml.py scripts/`
6. Update Justfile references

### Phase 5 — Crate relocation (optional, highest disruption)
Move `mml2vgm-rs/`, `mml2vgm-wasm/`, `egui-app/` → `crates/` and `browser-ide/` → `apps/browser-ide/`.  
**Hold this for a separate PR** — it requires updating `Justfile`, CI workflows, `wrangler.toml`, and any absolute paths baked into build scripts.

### Phase 6 — mml2vgmTest/ removal
Audit for anything not covered by `tests/`, then delete via `git rm -r mml2vgmTest/`.

---

## Notes

- **`docs/OPL3_Patch_Pack.mml`** and **`docs/Furnace_Instruments.mml`** — these are source files, not docs. Move them to `examples/` or `resources/`.
- **`docs/Furnace Tracks MML/`** is currently empty — delete.
- **`docs/PROJECT_STATUS.md`** — merge into `README.md` or move to `docs/dev/`.
- **Root `package.json`** only holds the Playwright dev dependency used by `smoke-test.mjs`. After moving the smoke test to `scripts/`, consider whether this belongs at root or consolidates into `browser-ide/package.json`.
- Phase 5 (crate relocation) is low-urgency — the current flat layout works fine; this is purely cosmetic.
