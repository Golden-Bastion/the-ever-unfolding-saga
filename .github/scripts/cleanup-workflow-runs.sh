#!/usr/bin/env bash
# Safely prune old GitHub Actions runs while retaining the current CI/CD run,
# the newest run of each companion workflow, and every queued/in-progress run.
set -uo pipefail

usage() {
    cat <<'USAGE'
Usage: cleanup-workflow-runs.sh [--dry-run] [--force]

Requires gh, jq, GH_TOKEN, and GITHUB_REPOSITORY (owner/repository).
By default, only successful CI/CD push runs on main or version tags (v*) may
trigger cleanup. --force permits an intentional manual run.
USAGE
}

dry_run=false
force=false
for arg in "$@"; do
    case "$arg" in
        --dry-run) dry_run=true ;;
        --force) force=true ;;
        -h|--help) usage; exit 0 ;;
        *) echo "Unknown argument: $arg" >&2; usage >&2; exit 2 ;;
    esac
done

for command in gh jq; do
    command -v "$command" >/dev/null 2>&1 || {
        echo "Required command not found: $command" >&2
        exit 2
    }
done
: "${GH_TOKEN:?GH_TOKEN must contain a GitHub token with Actions: write permission}"
: "${GITHUB_REPOSITORY:?GITHUB_REPOSITORY must be owner/repository}"

if [[ "$force" != true ]]; then
    [[ "${TRIGGER_CONCLUSION:-}" == success ]] || {
        echo "Skipping cleanup: triggering CI/CD run did not succeed."
        exit 0
    }
    [[ "${TRIGGER_EVENT:-}" == push ]] || {
        echo "Skipping cleanup: trigger was not a push."
        exit 0
    }
    branch="${TRIGGER_BRANCH:-}"
    [[ "$branch" == main || "$branch" == v* ]] || {
        echo "Skipping cleanup: push was neither main nor a version tag ($branch)."
        exit 0
    }
fi

api() {
    local attempt=1 max_attempts="${API_MAX_ATTEMPTS:-5}" delay=2
    while (( attempt <= max_attempts )); do
        if gh api "$@"; then
            return 0
        fi
        if (( attempt == max_attempts )); then
            echo "GitHub API request failed after $attempt attempts: gh api $*" >&2
            return 1
        fi
        echo "GitHub API request failed (attempt $attempt/$max_attempts); retrying in ${delay}s..." >&2
        sleep "$delay"
        ((attempt += 1))
        ((delay < 30)) && delay=$((delay * 2))
    done
}

workflow_names=("CI/CD" "PR Checks" "Android APK" "License check")
current_run_id="${CURRENT_RUN_ID:-}"
total_deleted=0
total_failed=0

echo "Repository: $GITHUB_REPOSITORY"
[[ "$dry_run" == true ]] && echo "DRY RUN: no workflow runs will be deleted."

for workflow_name in "${workflow_names[@]}"; do
    echo
    echo "=== $workflow_name ==="

    workflow_pages="$(api --paginate --slurp \
        "repos/${GITHUB_REPOSITORY}/actions/workflows?per_page=100")" || {
        echo "Unable to list workflows; continuing to the next workflow." >&2
        ((total_failed += 1))
        continue
    }

    workflow_id="$(jq -r --arg name "$workflow_name" \
        '[.[] | .workflows[]? | select(.name == $name)][0].id // empty' \
        <<<"$workflow_pages")"

    if [[ -z "$workflow_id" ]]; then
        echo "Workflow not found; continuing."
        continue
    fi

    run_pages="$(api --paginate --slurp \
        "repos/${GITHUB_REPOSITORY}/actions/workflows/${workflow_id}/runs?per_page=100")" || {
        echo "Unable to list runs for $workflow_name; continuing." >&2
        ((total_failed += 1))
        continue
    }

    mapfile -t runs < <(jq -r \
        '[.[] | .workflow_runs[]?] | sort_by(.created_at) | reverse | .[] | [.id, .status] | @tsv' \
        <<<"$run_pages")

    # Retain the current CI/CD run, or the newest run of each companion workflow.
    keep_run_id="$current_run_id"
    if [[ "$workflow_name" != "CI/CD" || -z "$keep_run_id" ]]; then
        for run in "${runs[@]}"; do
            IFS=$'\t' read -r candidate_id candidate_status <<<"$run"
            if [[ "$candidate_status" != completed ]]; then
                continue
            fi
            keep_run_id="$candidate_id"
            break
        done
    fi

    deleted=0
    failed=0
    for run in "${runs[@]}"; do
        IFS=$'\t' read -r run_id status <<<"$run"

        [[ -n "$run_id" ]] || continue
        [[ "$run_id" == "$keep_run_id" ]] && continue

        # Never cancel or delete runs that are still queued or executing.
        if [[ "$status" != completed ]]; then
            echo "Keeping active run $run_id (status: $status)."
            continue
        fi

        if [[ "$dry_run" == true ]]; then
            echo "Would delete completed run $run_id."
            continue
        fi

        echo "Deleting completed run $run_id..."
        if api --method DELETE "repos/${GITHUB_REPOSITORY}/actions/runs/${run_id}"; then
            ((deleted += 1))
        else
            echo "Failed to delete run $run_id; continuing." >&2
            ((failed += 1))
        fi
    done

    echo "$workflow_name: deleted $deleted, failed $failed."
    ((total_deleted += deleted))
    ((total_failed += failed))
done

echo
echo "Cleanup finished: deleted $total_deleted workflow runs; $total_failed operation(s) failed."
(( total_failed == 0 ))
