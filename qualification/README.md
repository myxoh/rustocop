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

A cop earns credit only when all of the following are true:

1. manual review is `passed` and names both source implementations;
2. the complete ported upstream diagnostic and correction contract passes;
3. at least four purpose-built edge cases pass differentially;
4. at least two provenance-backed real-world positives pass differentially;
5. at least two provenance-backed real-world negatives pass differentially.

`rustocop_commit` must be the exact commit containing the reviewed native code.
If code changes afterward, affected cops return to pending until all evidence is
rerun and the SHA is updated.

## Preparing a batch

Run the broad real-project gate before preparing per-cop evidence:

```sh
bundle exec ruby script/audit_qualification_projects.rb \
  --from-position 391 --count 30
```

The gate builds the release binary from a clean Rust tree, runs the selected
cops together across the pinned Chatwoot, RubyGems.org, and GitLab CE corpora,
and compares complete diagnostic signatures. It automatically isolates a
crashing cop so the remainder of the batch can finish. Its JSON and Markdown
reports classify every cop as `exact_active`, `dormant`, `mismatch`, or `crash`.

Only `exact_active` cops proceed to upstream, correction, evidence, and manual
source-boundary review. Dormant cops have not been exercised; mismatching and
crashing cops become separate implementation tasks. A project-exact result is
still only a candidate: manual comparison must probe semantic branches that the
real projects and captured upstream cases do not exercise.

Pass only those surviving cop names to the evidence preparer:

```sh
bundle exec ruby script/prepare_qualification_batch.rb \
  --cops Style/First,Style/Second
```

The preparer first runs every captured upstream diagnostic and correction case,
then scans the three pinned project corpora for real-world examples. It writes a
pending YAML record and a side-by-side review packet under `tmp/qualification/`.
Real-world candidates are retained only when RuboCop and Rustocop agree on both
diagnostics and the final autocorrected source. Repeated differential checks are
cached by candidate content and native binary digest.

Use `--cops Style/First,Style/Second` for an explicit batch or
`--from-position 451 --count 10` to resume at a matrix position. `--dry-run`
selects upstream edge cases and prints the record without building, scanning,
or writing files.

Generated records intentionally leave `manual_review.status` pending. Review
the paired source inventory, replace both TODO notes with concrete semantic
findings, and inspect all generated examples before moving the YAML into
`qualification/work/`. If the upstream contract fails, the review packet marks
the cop for Ruby-shaped callback conversion and a complete rerun.

The recorded Rust SHA is the most recent commit that changed `crates/rustocop`,
not merely the current repository HEAD. Preparation refuses to run while that
tree has staged or unstaged changes, preventing evidence from being pinned to a
commit that does not contain the tested native source.

Regenerate the README summary and the detailed per-cop ledger after adding or
repinning qualification records:

```sh
bundle exec ruby script/generate_qualification_progress.rb
```
