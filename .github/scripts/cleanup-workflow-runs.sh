#!/usr/bin/env bash
# Manually prune old GitHub Actions runs after a release has completed.
# Defaults to a dry run; use --apply only after reviewing the proposed deletions.
set -euo pipefail

usage() {
    cat <<'USAGE'
Usage: cleanup-workflow-runs.sh --release-tag vMAJOR.MINOR.PATCH [--dry-run | --apply] [--repo owner/repository]

Requires GitHub CLI (gh), jq, and a GitHub token with Actions: write permission.
The token may come from GH_TOKEN or an existing 'gh auth login' session.

The default is a dry run. Add --apply to delete completed runs.
The script retains the successful CI/CD and Android APK runs for the specified
release tag, the newest completed PR Checks and License check runs, and every
queued or in-progress run.
USAGE
}

release_tag=""
repository="${GITHUB_REPOSITORY:-}"
apply=false

while (($#)); do
    case "$1" in
        --release-tag)
            (($# >= 2)) || { echo "--release-tag requires a value." >&2; exit 2; }
            release_tag=$2
            shift 2
            ;;
        --repo)
            (($# >= 2)) || { echo "--repo requires a value." >&2; exit 2; }
            repository=$2
            shift 2
            ;;
        --apply)
            apply=true
            shift
            ;;
        --dry-run)
            apply=false
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo "Unknown argument: $1" >&2
            usage >&2
            exit 2
            ;;
    esac
done

if [[ ! "$release_tag" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "A valid --release-tag vMAJOR.MINOR.PATCH is required." >&2
    exit 2
fi

for command in gh jq; do
    command -v "$command" >/dev/null 2>&1 || {
        echo "Required command not found: $command" >&2
        exit 2
    }
done

if [[ -z "$repository" ]]; then
    repository="$(gh repo view --json nameWithOwner --jq '.nameWithOwner')" || {
        echo "Could not determine the repository. Run this inside its checkout or pass --repo owner/repository." >&2
        exit 2
    }
fi

if [[ ! "$repository" =~ ^[^/]+/[^/]+$ ]]; then
    echo "Invalid repository name: $repository (expected owner/repository)." >&2
    exit 2
fi

api() {
    local attempt=1
    local max_attempts="${API_MAX_ATTEMPTS:-5}"
    local delay="${API_RETRY_INITIAL_DELAY:-2}"

    if [[ ! "$max_attempts" =~ ^[1-9][0-9]*$ || ! "$delay" =~ ^[0-9]+$ ]]; then
        echo "API_MAX_ATTEMPTS must be positive and API_RETRY_INITIAL_DELAY non-negative." >&2
        return 2
    fi

    while ((attempt <= max_attempts)); do
        if gh api "$@"; then
            return 0
        fi
        if ((attempt == max_attempts)); then
            echo "GitHub API request failed after $attempt attempts: gh api $*" >&2
            return 1
        fi
        echo "GitHub API request failed (attempt $attempt/$max_attempts); retrying in ${delay}s..." >&2
        sleep "$delay"
        ((attempt += 1))
        if ((delay < 30)); then delay=$((delay * 2)); fi
    done
}

tag_ref="$(api "repos/$repository/git/ref/tags/$release_tag")" || {
    echo "Unable to resolve release tag $release_tag; nothing was deleted." >&2
    exit 1
}
object_type="$(jq -r '.object.type // empty' <<<"$tag_ref")"
object_sha="$(jq -r '.object.sha // empty' <<<"$tag_ref")"

for _ in 1 2 3 4 5; do
    case "$object_type" in
        commit) break ;;
        tag)
            tag_object="$(api "repos/$repository/git/tags/$object_sha")" || {
                echo "Unable to resolve annotated tag $release_tag; nothing was deleted." >&2
                exit 1
            }
            object_type="$(jq -r '.object.type // empty' <<<"$tag_object")"
            object_sha="$(jq -r '.object.sha // empty' <<<"$tag_object")"
            ;;
        *)
            echo "Release tag $release_tag does not resolve to a commit. Nothing was deleted." >&2
            exit 1
            ;;
    esac
done

if [[ "$object_type" != "commit" || ! "$object_sha" =~ ^[0-9a-f]{40}$ ]]; then
    echo "Could not resolve release tag $release_tag to a commit. Nothing was deleted." >&2
    exit 1
fi
release_sha="$object_sha"
echo "Release commit: $release_sha"

workflow_names=("CI/CD" "Android APK" "PR Checks" "License check")
declare -A workflow_ids run_pages keep_ids

echo "Repository: $repository"
echo "Release tag: $release_tag"
if [[ "$apply" == true ]]; then
    echo "Mode: APPLY — completed workflow runs will be deleted."
else
    echo "Mode: DRY RUN — no workflow runs will be deleted."
fi

# Complete discovery and safety checks before deleting anything.
workflow_pages="$(api --paginate --slurp \
    "repos/$repository/actions/workflows?per_page=100")" || {
    echo "Unable to list GitHub Actions workflows; nothing was deleted." >&2
    exit 1
}

for workflow_name in "${workflow_names[@]}"; do
    workflow_id="$(jq -r --arg name "$workflow_name" \
        '[.[] | .workflows[]? | select(.name == $name)][0].id // empty' \
        <<<"$workflow_pages")"

    if [[ -z "$workflow_id" ]]; then
        echo "Required workflow not found: $workflow_name. Nothing was deleted." >&2
        exit 1
    fi

    workflow_ids["$workflow_name"]=$workflow_id
    run_pages["$workflow_name"]="$(api --paginate --slurp \
        "repos/$repository/actions/workflows/$workflow_id/runs?per_page=100")" || {
        echo "Unable to list runs for $workflow_name; nothing was deleted." >&2
        exit 1
    }
done

for workflow_name in "${workflow_names[@]}"; do
    pages="${run_pages[$workflow_name]}"

    case "$workflow_name" in
        "CI/CD"|"Android APK")
            keep_id="$(jq -r --arg sha "$release_sha" '
                [.[] | .workflow_runs[]?
                    | select(.head_sha == $sha and .status == "completed" and .conclusion == "success")]
                | sort_by(.created_at) | last | .id // empty
            ' <<<"$pages")"
            if [[ -z "$keep_id" ]]; then
                echo "No successful completed $workflow_name run was found for release commit $release_sha. Nothing was deleted." >&2
                exit 1
            fi
            ;;
        *)
            keep_id="$(jq -r '
                [.[] | .workflow_runs[]? | select(.status == "completed")]
                | sort_by(.created_at) | last | .id // empty
            ' <<<"$pages")"
            ;;
    esac

    keep_ids["$workflow_name"]=$keep_id
    echo "Keeping $workflow_name run: ${keep_id:-none}"
done

total_deleted=0
total_planned=0
total_failed=0

for workflow_name in "${workflow_names[@]}"; do
    pages="${run_pages[$workflow_name]}"
    keep_id="${keep_ids[$workflow_name]}"
    workflow_id="${workflow_ids[$workflow_name]}"

    echo
    echo "=== $workflow_name (workflow $workflow_id) ==="

    mapfile -t runs < <(jq -r \
        '[.[] | .workflow_runs[]?] | sort_by(.created_at) | reverse | .[] | [.id, .status] | @tsv' \
        <<<"$pages")

    deleted=0
    planned=0
    failed=0

    for run in "${runs[@]}"; do
        IFS=$'\t' read -r run_id status <<<"$run"
        [[ -n "$run_id" ]] || continue
        [[ "$run_id" == "$keep_id" ]] && continue

        if [[ "$status" != "completed" ]]; then
            echo "Keeping active run $run_id (status: $status)."
            continue
        fi

        if [[ "$apply" != true ]]; then
            echo "Would delete completed run $run_id."
            ((planned += 1))
            continue
        fi

        echo "Deleting completed run $run_id..."
        if api --method DELETE "repos/$repository/actions/runs/$run_id"; then
            ((deleted += 1))
        else
            echo "Failed to delete run $run_id after retries; continuing." >&2
            ((failed += 1))
        fi
    done

    if [[ "$apply" == true ]]; then
        echo "$workflow_name: deleted $deleted, failed $failed."
    else
        echo "$workflow_name: would delete $planned, failed $failed."
    fi
    total_planned=$((total_planned + planned))
    total_deleted=$((total_deleted + deleted))
    total_failed=$((total_failed + failed))
done

echo
if [[ "$apply" == true ]]; then
    echo "Cleanup finished: deleted $total_deleted workflow runs; $total_failed deletion(s) failed."
else
    echo "Cleanup preview finished: $total_planned workflow runs would be deleted; no runs were deleted."
fi
((total_failed == 0))
