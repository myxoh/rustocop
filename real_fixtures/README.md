# Real fixtures

Each directory is an end-to-end compatibility contract:

- `input.rb` is the source presented to the linter.
- `rubocop.yml` is the complete RuboCop configuration for the example.
- `output.out` is RuboCop's `simple` formatter output for `input.rb`.
- `output.rb` is the result of running RuboCop with `--autocorrect-all`.

The first 20 numbered fixtures are deliberately small placeholders. Fixtures
from public projects add a `source.yml` containing the repository, license, PR,
commit, pre-fix revision, original path, and selected built-in cops.

## Adding a sourced fixture

1. Search merged Ruby PRs and commits whose titles mention RuboCop, linting, or
   fixing lint failures. Prefer changes tied to a failing CI job or a documented
   dirty RuboCop run.
2. Verify the repository's license from its committed license file and GitHub
   metadata. Only MIT-licensed sources belong in this directory.
3. Inspect the pre-fix `.rubocop.yml`, Gemfile, and patch. Reject examples that
   require project-defined cops or RuboCop extension gems.
4. Copy the smallest faithful pre-fix source region that still reproduces the
   offense. Do not rewrite the offending expression. Record any extraction in
   this README and pin both the source revision and fixing commit in `source.yml`.
5. Make `rubocop.yml` self-contained, disable unrelated cops, and enable only
   the upstream built-in cops under comparison.
6. Generate `output.out` and `output.rb` with RuboCop, then compare rustocop and
   RuboCop JSON diagnostics after normalizing tool metadata and paths. Run both
   autocorrectors and require the resulting Ruby source to be identical.

The sourced-fixture specs pass `--only` explicitly to both tools. This keeps the
contract about the named built-in cops and prevents unrelated default cops from
turning a focused upstream regression into a different test.

## Researched candidates

| Project and PR | License | Decision |
| --- | --- | --- |
| [`envato/zxcvbn-ruby#71`](https://github.com/envato/zxcvbn-ruby/pull/71) | MIT | Selected. The project used core RuboCop without extension gems, and the PR documented real offenses followed by a clean RuboCop run. |
| [`primer/view_components#1669`](https://github.com/primer/view_components/pull/1669) | MIT | Rejected because its configuration inherits `rubocop-github` and loads `rubocop/cop/primer`. |
| [`kjvarga/sitemap_generator#492`](https://github.com/kjvarga/sitemap_generator/pull/492) | MIT | Rejected for this pass because its lint setup loads the Performance, Rake, and RSpec RuboCop extensions. |
| [`dmee3/cap_ruby#212`](https://github.com/dmee3/cap_ruby/pull/212) | Unverified | Rejected because GitHub did not identify a repository license, despite the PR documenting a real lint failure. |

## `21_zxcvbn_ambiguous_ranges`

Source: [`lib/zxcvbn/matchers/date.rb`](https://github.com/envato/zxcvbn-ruby/blob/8a831434fa9019061a1c3a13fdc68fdd4be15656/lib/zxcvbn/matchers/date.rb), before
commit [`d851c0de83217b3ef518e94ef28979cfbbc5f960`](https://github.com/envato/zxcvbn-ruby/commit/d851c0de83217b3ef518e94ef28979cfbbc5f960)
from PR [#71](https://github.com/envato/zxcvbn-ruby/pull/71). The fixture keeps
the real `match_with_separator` method and its module/class nesting, while
omitting unrelated methods and constants. The repository's
[`LICENSE`](https://github.com/envato/zxcvbn-ruby/blob/master/LICENSE.txt) is MIT.

RuboCop 1.87.0 reports two correctable `Lint/AmbiguousRange` warnings and wraps
the arithmetic range boundaries. The first rustocop run exposed two defects:
the heuristic ignored nested/indented source, and an unrelated grouped-expression
cop could build a reversed byte range and panic. The implementation now trims
indentation before classifying the boundary, emits the same warning locations
and correction edits as RuboCop, and guards the grouped-expression range. The
fixture spec requires normalized JSON diagnostics and corrected source to match
RuboCop exactly.

Regenerate expected files after changing an input or configuration:

```sh
bundle exec ruby script/update_real_fixtures.rb
```

Pass one or more directory names to update only selected fixtures:

```sh
bundle exec ruby script/update_real_fixtures.rb 01_string_literals
```

Run `bundle exec rspec spec/real_fixtures_spec.rb` to verify that every fixture
still matches RuboCop 1.87.0.
