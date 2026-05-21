# Found ROMs Status — Terracren & Enduror

Two arcade ROM dumps shipped in this repository under `docs/terracren/` and
`docs/enduror/`. Both are **incomplete** and cannot be used to generate golden
masters for chip validation. They are retained for archival/historical reference
only.

For the broader validation workflow and external-ROM acquisition guidance, see
[`Validation_Status.md`](./Validation_Status.md), [`Golden_Master_Comparison_Plan.md`](./Golden_Master_Comparison_Plan.md),
and [`ROM_ACQUISITION_GUIDE.md`](./ROM_ACQUISITION_GUIDE.md).

---

## Inventory

| ROM | Path | Files | Size | Platform | Year | Sound chip |
|---|---|---|---|---|---|---|
| Enduro Racer | `docs/enduror/` | ~50 IC dumps | ~1.5 MB | Sega System 16 | 1986 | YM2151 (OPM) @ 4 MHz, confirmed via MAME `listxml` |
| Terrain Crumble (Terracren) | `docs/terracren/` | 22 IC dumps | 320 KB | Sega System 1 | 1987 | YM2203 (OPN), unverified |

Both are stored as raw `.rom` / `.bin` per-IC dumps rather than assembled MAME
ROM zips.

---

## Why they don't boot

MAME refuses to start either set because required IC files are missing from the
dumps:

```
mame enduror
> epr-7640a.ic97 NOT FOUND
> epr-7636a.ic84 NOT FOUND
> epr-7637.ic85   NOT FOUND
> ...
```

A complete Enduro Racer set would be roughly 1.5–2 MB of *correctly-named* ICs
matching the MAME dat. The files we have are partial.

Terracren has not been booted in MAME but is similarly partial.

---

## What to use instead

For YM2151 and YM2203 golden-master generation, acquire complete ROM sets from
external sources — see [`ROM_ACQUISITION_GUIDE.md`](./ROM_ACQUISITION_GUIDE.md).
MAME's `-wavwrite` flag has been confirmed working for capturing pristine audio
once a complete set is loaded.
