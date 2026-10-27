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

for source in agent_runtime.go docker_endpoint.go remote_access.go system_operations.go; do
  printf '%s\n' "${source}" > "${tmp_dir}/files"
  output="$("${selector}" --files-from "${tmp_dir}/files")"
  grep -Fq "QA contract consumer-compatibility:" <<< "${output}"
done

printf '%s\n' "power_test.go" > "${tmp_dir}/files"
output="$("${selector}" --files-from "${tmp_dir}/files")"
grep -Fq "QA contract wire-conformance:" <<< "${output}"
if grep -Fq "QA contract consumer-compatibility:" <<< "${output}"; then
  echo "test-only change unexpectedly selected downstream compatibility" >&2
  exit 1
fi

output="$("${selector}" --mode full --base '' --head HEAD)"
grep -Fq "QA contract wire-conformance:" <<< "${output}"
grep -Fq "QA contract consumer-compatibility:" <<< "${output}"

# Manual dispatch has no before/base SHA; it must still run every contract.
output="$(GITHUB_OUTPUT="${tmp_dir}/manual-output" "${selector}" --base '' --head HEAD)"
grep -Fxq 'mode=full' "${tmp_dir}/manual-output"
grep -Fxq 'checks_required=true' "${tmp_dir}/manual-output"
grep -Fq "QA contract consumer-compatibility:" <<< "${output}"

if "${selector}" --base >/dev/null 2>&1; then
  echo "missing --base argument was accepted" >&2
  exit 1
fi

output="$("${selector}" --base 0000000000000000000000000000000000000000 --head HEAD)"
grep -Fq "QA contract wire-conformance:" <<< "${output}"
grep -Fq "QA contract consumer-compatibility:" <<< "${output}"

printf 'broken\trow\n' > "${tmp_dir}/bad-manifest"
if "${selector}" --files-from "${tmp_dir}/files" --manifest "${tmp_dir}/bad-manifest" >/dev/null 2>&1; then
  echo "malformed QA manifest was accepted" >&2
  exit 1
fi

echo "QA contract selector tests passed"
