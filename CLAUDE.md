# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

- Build: `cargo build` (release: `cargo build --release`)
- Run: `cargo run --release` — reads `./data.csv` from the working directory (a ~17M CSV) and writes `./entities.csv` and `./records.csv`. Use release; the full run processes ~200k records.
- Test: `cargo test`
- Single test: `cargo test interior_bridge_with_two_real_endpoints_is_valid`
- Lint: `cargo clippy`; format: `cargo fmt`
- `justfile` shortcuts: `just test` (= `cargo t`), `just build` (= `cargo b --release`), `just run` (builds, then runs the release binary `./target/release/tc-rust`).

## What this does

This is an **entity-resolution / record-linkage** tool. It reads a CSV of organisation records and clusters records that refer to the same real-world entity by treating shared identifiers as edges in a graph, cutting weak single-record links, and counting the resulting connected components (= distinct entities).

The pipeline (`src/main.rs`) is:

1. **Parse** (`input.rs`) — each `Record` yields up to four identifier `(NodeKind, value)` pairs via `node_values()`: `group_id`, `account_number`, `abn`, `domain`. Values are canonicalised (whitespace stripped, lowercased, `NULL`/empty → `None`); ABNs are checksum-validated (`validate_abn`, mod-89, hardened against non-digits / leading zero) and generic email domains dropped.
2. **Intern** (`node.rs`) — every distinct identifier is interned once to a dense `NodeId` (`u32`) via `Interner`. **All downstream work uses integer ids and `Vec`-indexed state**, not `String`-keyed maps. This is the core performance decision; preserve it.
3. **Build graph** (`graph.rs`) — `GraphBuilder::add_clique` makes each record's ids a weighted clique, accumulating each undirected edge's weight (= number of records co-asserting that pair) in an `FxHashMap<(NodeId,NodeId),u32>`, then `build()` freezes it into CSR-style adjacency `Vec<Vec<(NodeId, weight)>>`.
4. **Find valid bridges** (`bridge_finder.rs`) — `find_valid_bridges` runs an **iterative** Tarjan low-link DFS (explicit `Vec` stack, single global discovery counter). A bridge is kept only if **valid**: `weight == 1` (single-record link) **and** both endpoints have `degree > 1` (so cutting separates two real clusters rather than orphaning a leaf).
5. **Count components** (`union_find.rs`) — `UnionFind` (path-halving + union-by-size) is initialised over **all** interned ids, then unions every edge except the cut bridges. `main` reports the entity count before vs. after cutting (after − before == number of bridges cut).
6. **Write clusters** (`output.rs`) — `entities.csv` (`entity_id, kind, value`, one row per identifier) and `records.csv` (`record_id, entity_id, split`; a record is assigned to the entity holding the majority of its identifiers, `split = true` when its identifiers span >1 entity — only possible for a record whose linking edge was cut).

## Architecture notes

- **`NodeId` (`u32`) is the universal currency.** `Interner` (`node.rs`) maps `(NodeKind, value)` → id and back. `NodeKind` is part of identity, so the same string under two kinds is two distinct nodes. Avoid reintroducing `String`-keyed graph/union-find state — that was the v1 bottleneck.
- **Graph is build-then-freeze and read-only.** Mutate edges only through `GraphBuilder`; `Graph` itself exposes `neighbors`, `degree`, `edges()`.
- **Bridge finding is iterative on purpose** — recursion would stack-overflow on deep components in this dataset. Keep the explicit stack and the global `timer` (not a depth/parameter-based counter).
- **Counting starts from all ids, including isolated ones.** A record with a single valid identifier is a real (isolated) entity and is counted — do not regress to counting only nodes that have edges.

## Data dependency

`cargo run` requires `data.csv` in the repo root with columns including `Id, group_id, abn, account_number, email_domain_name, generic_domain` (the file has a UTF-8 BOM; the `csv` crate handles it). `generic_domain` is read as `1`/`0` (`bool_from_string`). The program errors on `File::open` if it is missing. Records with no valid identifiers are skipped (reported in the run summary).
