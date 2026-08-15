# Nuclide catalogue

The Radiacode Spectrum application ships with a bundled nuclide catalogue derived from the [IAEA Livechart of Nuclides](https://nds.iaea.org/relnsd/vcharthtml/VChartHTML.html) / ENSDF database.

## Data provenance

- **Source:** IAEA Livechart REST API (`https://nds.iaea.org/relnsd/v1/data`)
- **Fields used:** `ground_states` (half-life, decay modes) and `decay_rads` with `rad_types=g` (gamma) and `rad_types=x` (X-ray)
- **Attribution:** IAEA Nuclear Data Section, ENSDF evaluators

## Selection rules

Nuclides are included when they meet any of:

- Half-life ≥ 1 second and at least one gamma line with intensity ≥ 0.1% and energy ≤ 4000 keV
- Member of a natural decay chain (U-238, U-235, Th-232, Np-237) force-included

Gamma lines stored per nuclide are filtered to intensity ≥ 0.1% and energy ≤ 4000 keV.

## Regenerating the dataset

The importer is feature-gated and not used in normal builds:

```bash
cargo run -p radiacode-nuclides --bin nuclide-import --features import
```

Output is written to `radiacode-nuclides/data/nuclides.json` and embedded at compile time.

Optional cap for development:

```bash
cargo run -p radiacode-nuclides --bin nuclide-import --features import -- --limit=500
```

## Peak identification

Detected peaks are matched against catalogue gamma lines using energy-dependent tolerance:

```
tolerance_kev = max(energy_kev × relative_frac, floor_kev)
```

Default parameters (Settings → Application → Isotope matching):

| Parameter | Default |
|-----------|---------|
| Relative tolerance | 1% |
| Floor | 3 keV |
| Min gamma intensity | 1% |

Scoring combines line intensity, energy closeness, and multi-line confirmation across all detected peaks in the current view.

## Crate layout

```
radiacode-nuclides/
  data/nuclides.json     bundled dataset
  src/catalog.rs         lazy JSON parse
  src/index.rs           energy-sorted lookup
  src/match_peaks.rs     identification scoring
  src/chain.rs           decay chain traversal
  src/search.rs          catalogue browser filters
```
