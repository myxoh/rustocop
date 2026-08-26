# Supplemental cop qualification records

This directory preserves the historical five-check records used for the
original 60-cop qualification batch. The retired qualification runner has been
removed; these records are not current public support evidence. Current support
is determined by the complete fixture and 50-project snapshots documented in
the root README and `docs/compatibility.md`.

Historical `verified` or `heuristic` labels do not grant compatibility credit.
The current controlled fixture evidence was refreshed at
`2026-08-26T16:18:45-04:00`; the exact 50-project evidence was refreshed at
`2026-08-26T16:07:54-04:00`.

The first qualification batch is the first 60 entries in the complete RuboCop
1.87 built-in matrix. Work files use this schema:

```yaml
schema: 1
batch: short_batch_name
rubocop_version: 1.87.0
rubocop_commit: e5b788dba181ad94de30cfbad661c5d6aa08a4e5
rustocop_commit: pending
cops:
  Department/CopName:
    matrix_position: 1
    sources:
      rubocop: lib/rubocop/cop/department/cop_name.rb
      rustocop:
        - crates/rustocop/src/cops/prism/example.rs
    manual_review:
      status: pending # pending, passed, or failed
      notes:
        - Specific behavior compared between the implementations.
    upstream_tests:
      status: pending # pending, passed, or failed
      passed: 0
      total: 0
      corrections: false
    edge_cases:
      - id: descriptive_unique_id
        description: Why this is an edge case.
        path: example.rb
        config: {}
        source: |
          example
    real_world:
      positives:
        - repository: owner/name
          revision: full_git_sha
          path: path/in/repository.rb
          line: 1
          source: |
            offending_example
      negatives:
        - repository: owner/name
          revision: full_git_sha
          path: path/in/repository.rb
          line: 1
          source: |
            non_offending_example
```

Each recorded edge and real-world case was required to be executable Ruby
source. The historical differential runner executed only the named cop with the
recorded config and required Rustocop's normalized diagnostics to match RuboCop
exactly. A real positive had to produce at least one RuboCop offense; a real
negative had to produce none.

A cop earns credit only when all of the following are true:

1. manual review is `passed` and names both source implementations;
2. the complete ported upstream diagnostic and correction contract passes;
3. at least four purpose-built edge cases pass differentially;
4. at least two provenance-backed real-world positives pass differentially;
5. at least two provenance-backed real-world negatives pass differentially.

`rustocop_commit` must be the exact commit containing the reviewed native code.
If code changes afterward, affected cops return to pending until all evidence is
rerun and the SHA is updated.
