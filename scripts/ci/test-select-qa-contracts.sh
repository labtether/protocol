#!/usr/bin/env bash
set -Eeuo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
selector="${repo_root}/scripts/ci/select-qa-contracts.sh"
tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/labtether-qa-selector-test.XXXXXX")"
cleanup() {
  rm -rf -- "${tmp_dir}"
}
trap cleanup EXIT

printf '%s\n' "message.go" > "${tmp_dir}/files"
output="$("${selector}" --files-from "${tmp_dir}/files")"
grep -Fq "QA contract wire-conformance:" <<< "${output}"
grep -Fq "QA contract consumer-compatibility:" <<< "${output}"

printf '%s\n' "power_test.go" > "${tmp_dir}/files"
output="$("${selector}" --files-from "${tmp_dir}/files")"
grep -Fq "QA contract wire-conformance:" <<< "${output}"
if grep -Fq "QA contract consumer-compatibility:" <<< "${output}"; then
  echo "test-only change unexpectedly selected downstream compatibility" >&2
  exit 1
fi

output="$("${selector}" --mode full)"
grep -Fq "QA contract wire-conformance:" <<< "${output}"
grep -Fq "QA contract consumer-compatibility:" <<< "${output}"

output="$("${selector}" --base 0000000000000000000000000000000000000000 --head HEAD)"
grep -Fq "QA contract wire-conformance:" <<< "${output}"
grep -Fq "QA contract consumer-compatibility:" <<< "${output}"

printf 'broken\trow\n' > "${tmp_dir}/bad-manifest"
if "${selector}" --files-from "${tmp_dir}/files" --manifest "${tmp_dir}/bad-manifest" >/dev/null 2>&1; then
  echo "malformed QA manifest was accepted" >&2
  exit 1
fi

echo "QA contract selector tests passed"
