#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
script="$repo_root/.github/scripts/cleanup-workflow-runs.sh"
test_dir="$(mktemp -d)"
trap 'rm -rf "$test_dir"' EXIT
mkdir -p "$test_dir/bin"

fail() {
    echo "FAIL: $*" >&2
    exit 1
}

bash -n "$script"
bash -n "$repo_root/.github/scripts/extract-release-notes.sh"

cat > "$test_dir/bin/gh" <<'MOCK_GH'
#!/usr/bin/env bash
set -euo pipefail

if [[ "${1:-}" == "repo" && "${2:-}" == "view" ]]; then
    echo "example/repo"
    exit 0
fi

[[ "${1:-}" == "api" ]] || {
    echo "Unexpected gh command: $*" >&2
    exit 2
}

args="$*"

if [[ "$args" == *"actions/workflows?per_page=100"* ]]; then
    echo '[{"workflows":[{"id":1,"name":"CI/CD"},{"id":2,"name":"Android APK"},{"id":3,"name":"PR Checks"},{"id":4,"name":"License check"}]}]'
    exit 0
fi

if [[ "$args" == *"/actions/workflows/1/runs?per_page=100"* ]]; then
    echo '[{"workflow_runs":[
      {"id":101,"head_branch":"v0.53.0","status":"completed","conclusion":"success","created_at":"2026-11-20T12:00:00Z"},
      {"id":102,"head_branch":"main","status":"completed","conclusion":"success","created_at":"2026-11-19T12:00:00Z"},
      {"id":103,"head_branch":"main","status":"in_progress","conclusion":null,"created_at":"2026-11-18T12:00:00Z"}
    ]}]'
    exit 0
fi

if [[ "$args" == *"/actions/workflows/2/runs?per_page=100"* ]]; then
    echo '[{"workflow_runs":[
      {"id":201,"head_branch":"v0.53.0","status":"completed","conclusion":"success","created_at":"2026-11-20T12:01:00Z"},
      {"id":202,"head_branch":"v0.52.70","status":"completed","conclusion":"success","created_at":"2026-11-19T12:01:00Z"}
    ]}]'
    exit 0
fi

if [[ "$args" == *"/actions/workflows/3/runs?per_page=100"* ]]; then
    echo '[{"workflow_runs":[
      {"id":301,"head_branch":"feature/example","status":"completed","conclusion":"success","created_at":"2026-11-20T12:02:00Z"},
      {"id":302,"head_branch":"main","status":"completed","conclusion":"failure","created_at":"2026-11-19T12:02:00Z"}
    ]}]'
    exit 0
fi

if [[ "$args" == *"/actions/workflows/4/runs?per_page=100"* ]]; then
    echo '[{"workflow_runs":[
      {"id":401,"head_branch":"main","status":"completed","conclusion":"success","created_at":"2026-11-20T12:03:00Z"},
      {"id":402,"head_branch":"main","status":"completed","conclusion":"success","created_at":"2026-11-19T12:03:00Z"}
    ]}]'
    exit 0
fi

if [[ "$args" == *"--method DELETE"* ]]; then
    run_path="${!#}"
    run_id="${run_path##*/}"
    echo "$run_id" >> "$ATTEMPT_LOG"

    if [[ "$run_id" == "102" && ! -f "$RETRY_MARKER" ]]; then
        touch "$RETRY_MARKER"
        echo "HTTP 502 simulated for regression test." >&2
        exit 1
    fi

    echo "$run_id" >> "$DELETE_LOG"
    exit 0
fi

echo "Unexpected gh api request: $args" >&2
exit 2
MOCK_GH
chmod +x "$test_dir/bin/gh"

export PATH="$test_dir/bin:$PATH"
export GITHUB_REPOSITORY="example/repo"
export DELETE_LOG="$test_dir/deleted"
export ATTEMPT_LOG="$test_dir/attempts"
export RETRY_MARKER="$test_dir/retry-once"
export API_MAX_ATTEMPTS=2
export API_RETRY_INITIAL_DELAY=0

bash "$script" --release-tag v0.53.0 > "$test_dir/dry-run.log"
grep -Fq 'Mode: DRY RUN' "$test_dir/dry-run.log" || fail "dry run is not the default"
grep -Fq 'Would delete completed run 102.' "$test_dir/dry-run.log" || fail "older CI/CD run was not selected"
grep -Fq 'Keeping active run 103' "$test_dir/dry-run.log" || fail "active workflow run was not retained"
[[ ! -f "$DELETE_LOG" ]] || fail "dry run deleted a workflow run"

bash "$script" --release-tag v0.53.0 --apply > "$test_dir/apply.log"
for run_id in 102 202 302 402; do
    grep -Fxq "$run_id" "$DELETE_LOG" || fail "expected old run $run_id to be deleted"
done
for run_id in 101 103 201 301 401; do
    ! grep -Fxq "$run_id" "$DELETE_LOG" || fail "protected run $run_id was deleted"
done

[[ "$(grep -Fc '102' "$ATTEMPT_LOG")" -eq 2 ]] \
    || fail "a transient 502 response was not retried"

if bash "$script" --release-tag invalid > /dev/null 2>&1; then
    fail "an invalid release tag was accepted"
fi

echo "Manual workflow cleanup script tests passed."
