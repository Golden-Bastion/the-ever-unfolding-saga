#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
script="$repo_root/.github/scripts/cleanup-workflow-runs.sh"
workflow="$repo_root/.github/workflows/clean-up.yml"

bash -n "$script"

grep -Fq 'workflow_run:' "$workflow"
grep -Fq 'bash .github/scripts/cleanup-workflow-runs.sh' "$workflow"
grep -Fq 'TRIGGER_BRANCH:' "$workflow"
grep -Fq 'TRIGGER_CONCLUSION:' "$workflow"

grep -Fq 'API_MAX_ATTEMPTS' "$script"
grep -Fq 'sleep "$delay"' "$script"
grep -Fq 'continue' "$script"
grep -Fq 'status" != completed' "$script"
grep -Fq 'keep_run_id' "$script"
grep -Fq -- '--dry-run' "$script"
grep -Fq -- '--force' "$script"

echo "Workflow cleanup script checks passed."
