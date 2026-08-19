# Lint/ScriptPermission

`Lint/ScriptPermission` cannot yet be verified faithfully with the captured
upstream corpus. The cop depends on filesystem metadata and reports the source
file's basename; the current corpus stores only source text and the comparison
runner recreates every case under a different temporary filename and mode.

Implementing this as a source-text heuristic would be misleading. Correct
support needs all of the following:

- capture the original executable/non-executable mode as case metadata;
- make the comparison runner recreate that mode and provide a stable basename;
- expose filesystem metadata to cops without weakening stdin isolation;
- treat autocorrection as a permission change (`chmod`), not a text edit, and
  test that state transition separately from source correction.

The runtime check itself is straightforward once that infrastructure exists.
Until then, Rustocop deliberately skips this cop rather than reporting every
shebang encountered through stdin as a false positive.
