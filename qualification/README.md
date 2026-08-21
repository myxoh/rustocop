# Cop qualification evidence

This directory is the authoritative evidence source for the five qualification
checks summarized in the project README. Historical `verified` or `heuristic`
labels do not grant qualification credit.

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

Each edge and real-world case must be executable Ruby source. The differential
qualification runner will execute only the named cop with the recorded config
and require Rustocop's normalized diagnostics to match RuboCop exactly. A real
positive must produce at least one RuboCop offense; a real negative must
produce none.

Run only the edge-case fixture check while a batch is being assembled with:

```sh
bundle exec ruby script/verify_qualification.rb --no-upstream --checks 3
```

`RUSTOCOP_NATIVE_PATH` can point the verifier at a worktree build without
overwriting the checked-in native executable.

A cop earns credit only when all of the following are true:

1. manual review is `passed` and names both source implementations;
2. the complete ported upstream diagnostic and correction contract passes;
3. at least four purpose-built edge cases pass differentially;
4. at least two provenance-backed real-world positives pass differentially;
5. at least two provenance-backed real-world negatives pass differentially.

`rustocop_commit` must be the exact commit containing the reviewed native code.
If code changes afterward, affected cops return to pending until all evidence is
rerun and the SHA is updated.
