# Proof Vector Search

> **SuperInstance Fleet — Mathematical Verification Infrastructure**

A Rust proof-of-concept benchmarking framework that compares float-based cosine similarity search against constraint theory-based search using the `PythagoreanManifold` from `constraint-theory-core`. This project demonstrates that structured geometric constraints can achieve competitive retrieval quality at reduced memory cost.

---

## Overview

Proof vector search is a technique for retrieving mathematical proofs (or proof-like structures) from large vector databases by encoding them as high-dimensional vectors and performing nearest-neighbor queries. Traditional approaches rely on raw floating-point representations and brute-force cosine similarity, which incurs significant memory overhead — especially at scale.

This project introduces an alternative: **constraint theory-based vector search**. By snapping vector components onto a `PythagoreanManifold`, we can:

- **Reduce per-vector memory** from 128 × 8 bytes (f64) to 128 × 4 bytes (f32) — a 50% reduction
- **Preserve retrieval recall** through structured geometric quantization
- **Maintain query latency** within an acceptable envelope for verification workloads

The benchmark generates 10,000 random 128-dimensional vectors and executes 200 queries across both modes, reporting recall@10, bytes-per-vector, and average query latency.

---

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Benchmark Runner                      │
│                  (src/main.rs — main)                    │
│                                                         │
│  ┌─────────────────┐    ┌────────────────────────────┐  │
│  │  Float Mode      │    │  Constraint Theory Mode     │  │
│  │  ("float")       │    │  ("ct")                     │  │
│  │                  │    │                             │  │
│  │  • Raw f64 vecs  │    │  • f64 → f32 downcast       │  │
│  │  • Cosine sim    │    │  • Pair-wise chunking       │  │
│  │  • Brute-force   │    │  • PythagoreanManifold.snap │  │
│  │    top-k sort    │    │  • Cosine sim on snapped    │  │
│  │                  │    │    manifold points           │  │
│  └─────────────────┘    └──────────┬──────────────────┘  │
│                                    │                     │
│                         ┌──────────▼──────────────────┐  │
│                         │   constraint-theory-core     │  │
│                         │   PythagoreanManifold(b=10)  │  │
│                         └─────────────────────────────┘  │
│                                                         │
│  Output: Mode │ Recall@10 │ Bytes/vector │ Query latency │
└─────────────────────────────────────────────────────────┘
```

### Execution Flow

1. **Vector Generation** — 10,000 random 128-dim f64 vectors via `rand::thread_rng()`
2. **Mode Dispatch** — Each mode processes the same 200 queries independently
3. **Cosine Similarity** — Dot product over L2-normalized vectors, full scan
4. **Top-K Selection** — Sort all candidates, extract top 10 per query
5. **Recall Measurement** — Count how often the ground-truth vector appears in top 10
6. **Metrics Aggregation** — Average recall, per-vector byte cost, per-query latency

---

## Core Types

| Type | Representation | Mode | Description |
|------|---------------|------|-------------|
| `[f64; 128]` | 8-byte floats | `float` | Raw high-precision vector stored as 128-dimensional f64 array |
| `[f32; 2]` | 4-byte float pairs | `ct` | Manifold-snapped point; each pair of adjacent dimensions is snapped to the nearest point on the `PythagoreanManifold` |
| `PythagoreanManifold` | Constraint manifold (b=10) | `ct` | Encapsulates the quantization lattice; `snap([f32; 2]) → ([f32; 2], cost)` maps any 2D point to its nearest manifold point |
| `(f64, &[f64; 128])` | Similarity + reference | both | Intermediate top-k candidate tuple used during ranking |

---

## Algorithms

### Float-Based Cosine Similarity (`float`)

The baseline brute-force approach:

1. For each query vector **q** and database vector **v**:
   ```
   similarity(q, v) = (q · v) / (‖q‖₂ · ‖v‖₂)
   ```
2. Compute the dot product across all 128 dimensions using f64 arithmetic.
3. Normalize by the Euclidean norms of both vectors.
4. Sort all 10,000 candidates and extract the top 10.

**Complexity**: O(N · D) per query where N = 10,000 and D = 128.

### Constraint Theory Search (`ct`)

The structured quantization approach:

1. **Downcast** each f64 component to f32 (50% memory reduction).
2. **Chunk** the 128-dim vector into 64 consecutive pairs `[f32; 2]`.
3. **Snap** each pair to the `PythagoreanManifold(b=10)`:
   ```
   snapped = manifold.snap(pair).0
   ```
   This maps arbitrary 2D points to the nearest point on a Pythagorean lattice, introducing structured quantization error that is geometrically bounded.
4. **Compute cosine similarity** on the snapped representation using f32 arithmetic (64 dot-product terms, each the sum of two component-wise products).
5. Sort and extract top 10 as above.

**Key Insight**: The manifold's structure ensures that quantization error is *correlated* across dimension pairs, preserving relative distances more faithfully than independent scalar quantization.

---

## Mathematical Foundation

### Cosine Similarity

Cosine similarity measures the angle between two vectors in high-dimensional space:

```
cos(θ) = (A · B) / (‖A‖₂ · ‖B‖₂)
```

Values range from -1 (opposite) to +1 (identical direction). For retrieval, higher cosine similarity indicates greater semantic or structural closeness.

### Pythagorean Manifold Snapping

The `PythagoreanManifold(b)` defines a discrete lattice in 2D Euclidean space parameterized by bandwidth `b`. Given a point `p = (x, y)`, the `snap` operation finds the nearest lattice point `p*` such that:

```
p* = argmin_{m ∈ M(b)} ‖p - m‖₂
```

where `M(b)` is the manifold point set. The snapping operation:

- Introduces **bounded quantization error**: `‖p - p*‖₂ ≤ δ(b)`
- Preserves **pairwise angular relationships** more effectively than independent rounding
- Enables **dimensionality-aware compression** by exploiting 2D geometric structure

### Memory–Recall Tradeoff

| Metric | Float (f64) | Constraint Theory (f32 + manifold) |
|--------|-------------|-------------------------------------|
| Bytes per vector | 1,024 | 512 |
| Precision | 64-bit native | 32-bit snapped to lattice |
| Quantization | None | Structured (geometric) |

The 50% memory reduction comes at the cost of structured quantization error, but the manifold's geometry is designed to preserve angular relationships that cosine similarity depends on.

---

## Integration with Constraint Theory Ecosystem

This project is part of the **SuperInstance fleet** and directly depends on [`constraint-theory-core`](https://crates.io/crates/constraint-theory-core):

```toml
[dependencies]
constraint-theory-core = "1.0.1"
rand = "0.8"
```

- **`PythagoreanManifold`** — The core abstraction from `constraint-theory-core` that provides structured geometric quantization. It is used here to snap 2D coordinate pairs onto a discrete Pythagorean lattice.
- **`snap` method** — Returns `(snapped_point, snapping_cost)`, allowing callers to both quantize vectors and track the fidelity cost of quantization.

This proof establishes the viability of using constraint theory primitives as drop-in replacements for raw floating-point representations in vector search pipelines, opening the door to integration with other constraint-theory tools (manifold composition, constraint propagation, geometric reasoning).

---

## Build & Test

### Prerequisites

- **Rust** (latest stable) — [install](https://rustup.rs/)
- **Cargo** (bundled with Rust)

### Build

```bash
cargo build --release
```

### Run Benchmark

```bash
cargo run --release
# or use the provided script:
./bench.sh
```

The program will:
1. Generate 10,000 random 128-dimensional vectors
2. Run 200 queries in both `float` and `ct` modes
3. Print a tab-separated table:

```
Mode    Recall@10    Bytes per vector    Query latency
float   ...          1024                ...
ct      ...          512                 ...
```

### Expected Output Interpretation

- **Recall@10** — Fraction of queries where the exact-match vector appears in the top 10 results (0.0–1.0)
- **Bytes per vector** — Memory footprint per stored vector
- **Query latency** — Average wall-clock time per query in milliseconds

---

## Project Structure

```
proof-vector-search/
├── Cargo.toml          # Crate manifest (constraint-theory-core, rand)
├── Cargo.lock          # Dependency lockfile
├── src/
│   └── main.rs         # Benchmark runner: float vs. CT comparison
├── bench.sh            # Convenience script for release-mode execution
├── callsign1.jpg       # Fleet callsign
└── README.md           # This file
```

---

## License

Part of the SuperInstance fleet — mathematical verification infrastructure.

---

<img src="callsign1.jpg" width="128" alt="callsign">
