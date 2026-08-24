# Project-owned fixtures

This directory is reserved for fixtures whose contract is a whole project's
output or a cross-cop interaction. Put committed data under
`projects/<project-name>/` and record its repository and pinned revision there.

Do not put a minimized single-cop regression here. Even when its source came
from a real project, it belongs under `../cops/<Department>/<Cop>/project/` and
in `../cop_project_cases.tsv`.

The reproducible 50-project source trees and configuration variations are
transient development probes, intentionally cached outside Git under
`tmp/project-benchmarks/corpora/`. They stay out of the millisecond unit cycle
and may eventually be discarded after their useful differences have all been
minimized into controlled cop fixtures. The aggregate RuboCop result snapshot
remains compatibility evidence under `spec/compatibility_evidence/`.
