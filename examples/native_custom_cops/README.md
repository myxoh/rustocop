# Native custom-cop examples

This directory is an isolated authoring corpus for cops supplied by RuboCop
extensions. These cops are intentionally separate from the 606 built-in
RuboCop compatibility fixtures and their public evidence denominator.

The first pack translates five cops from `rubocop-performance` 1.26.1. Its
controlled examples live in `rubocop-performance-1.26.1/cases.yml`; generated
contracts cache exact diagnostics plus safe (`-a`) and all (`-A`) corrections.

Refresh the third-party oracle only when its input cases or pinned version
change:

```sh
bundle exec ruby script/capture_extension_cop_examples.rb \
  examples/native_custom_cops/rubocop-performance-1.26.1/cases.yml
```

The normal edit/test loop does not launch Ruby or RuboCop:

```sh
bundle exec ruby script/verify_extension_cops.rb \
  examples/native_custom_cops/rubocop-performance-1.26.1 \
  Performance/ReverseEach Performance/ReverseFirst Performance/Size \
  Performance/StringBytesize Performance/RedundantSortBlock
```

Pass `--refresh` to recapture and then verify. Keep the YAML cases small and
intentional: offending forms, adjacent clean controls, configuration branches,
and correction boundaries are more valuable than copied project files.
