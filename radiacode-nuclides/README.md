# radiacode-nuclides

Bundled nuclide catalogue with gamma lines, decay topology, chain graphs, and peak-matching for isotope identification.

Data is embedded at compile time from `data/nuclides.json`. Decay topology ships in `data/decays.json`.

## Features

- Lazy catalogue parse and energy-sorted gamma index
- Decay chain traversal, branching graphs, and equilibrium weights
- Natural decay series (U-238, U-235, Th-232, Np-237)
- Resolution-aware peak matching with multi-line scoring
- Source summary grouping matches into nuclides and series

## Usage

```toml
radiacode-nuclides = { path = "../radiacode-nuclides" }
```

```rust
use radiacode_nuclides::{match_peaks, nuclide_by_id, MatchParams, SpectrumPeak};

let peaks = vec![SpectrumPeak { energy_kev: 661.7, intensity: 1.0 }];
let params = MatchParams::default();
let matches = match_peaks(&peaks, &params);
```

## Regenerating data

The importer is feature-gated:

```bash
cargo run -p radiacode-nuclides --bin nuclide-import --features import
```

See [docs/NUCLIDES.md](../docs/NUCLIDES.md) for provenance, selection rules, and matching parameters.

## Tests

```bash
cargo test -p radiacode-nuclides
```

## License

AGPL-3.0-only — see [LICENSE](../LICENSE).
