#!/usr/bin/env bash
#
# Push the current branch to every configured remote.
#
# Usage:
#   ./scripts/push-all.sh              push to all remotes
#   ./scripts/push-all.sh github       push to a single remote
#   ./scripts/push-all.sh --tags       also push tags
#   ./scripts/push-all.sh --dry-run    show what would happen
#
# Configuration:
#   Set REMOTES below, or override with an environment variable:
#     REMOTES="github codeberg" ./scripts/push-all.sh

set -euo pipefail

# -----------------------------------------------------------------------------
# Configuration
# -----------------------------------------------------------------------------

# Remotes to push to, in order.
REMOTES_DEFAULT="origin codeberg"

# Allow override via environment.
REMOTES="${REMOTES:-$REMOTES_DEFAULT}"

# -----------------------------------------------------------------------------
# Colors
# -----------------------------------------------------------------------------

if [ -t 1 ]; then
    RED=$'\033[0;31m'
    GREEN=$'\033[0;32m'
    YELLOW=$'\033[0;33m'
    BLUE=$'\033[0;34m'
    BOLD=$'\033[1m'
    RESET=$'\033[0m'
else
    RED=""
    GREEN=""
    YELLOW=""
    BLUE=""
    BOLD=""
    RESET=""
fi

log()   { printf '%s==>%s %s\n' "$BLUE" "$RESET" "$*"; }
ok()    { printf '%s ok %s %s\n' "$GREEN" "$RESET" "$*"; }
warn()  { printf '%swarn%s %s\n' "$YELLOW" "$RESET" "$*" >&2; }
die()   { printf '%s error %s %s\n' "$RED" "$RESET" "$*" >&2; exit 1; }

# -----------------------------------------------------------------------------
# Argument parsing
# -----------------------------------------------------------------------------

PUSH_TAGS=0
DRY_RUN=0
REMOTES_ARG=""

while [ $# -gt 0 ]; do
    case "$1" in
        --tags)
            PUSH_TAGS=1
            shift
            ;;
        --dry-run|-n)
            DRY_RUN=1
            shift
            ;;
        -h|--help)
            sed -n '2,20p' "$0"
            exit 0
            ;;
        -*)
            die "unknown flag: $1"
            ;;
        *)
            if [ -n "$REMOTES_ARG" ]; then
                die "only one remote can be specified"
            fi
            REMOTES_ARG="$1"
            shift
            ;;
    esac
done

if [ -n "$REMOTES_ARG" ]; then
    REMOTES="$REMOTES_ARG"
fi

# -----------------------------------------------------------------------------
# Preflight checks
# -----------------------------------------------------------------------------

log "checking repository state"

git rev-parse --is-inside-work-tree >/dev/null 2>&1 \
    || die "not inside a git repository"

BRANCH="$(git rev-parse --abbrev-ref HEAD)"
[ "$BRANCH" != "HEAD" ] || die "detached HEAD; checkout a branch first"

if [ -n "$(git status --porcelain)" ]; then
    warn "working tree has uncommitted changes"
    warn "they will NOT be pushed"
fi

# -----------------------------------------------------------------------------
# Push
# -----------------------------------------------------------------------------

log "branch: $BRANCH"
log "remotes: $REMOTES"

# Resolve the list of remotes that actually exist.
EXISTING_REMOTES="$(git remote)"
PUSHED=0
FAILED=0

for remote in $REMOTES; do
    if ! printf '%s\n' "$EXISTING_REMOTES" | grep -Fxq "$remote"; then
        warn "remote '$remote' is not configured; skipping"
        FAILED=$((FAILED + 1))
        continue
    fi

    log "pushing to '$remote'"

    if [ "$DRY_RUN" -eq 1 ]; then
        printf '    git push %s %s\n' "$remote" "$BRANCH"
        [ "$PUSH_TAGS" -eq 1 ] && printf '    git push %s --tags\n' "$remote"
        PUSHED=$((PUSHED + 1))
        continue
    fi

    if git push "$remote" "$BRANCH"; then
        ok "$remote"
        PUSHED=$((PUSHED + 1))
    else
        warn "push to '$remote' failed"
        FAILED=$((FAILED + 1))
    fi

    if [ "$PUSH_TAGS" -eq 1 ]; then
        log "pushing tags to '$remote'"
        if git push "$remote" --tags; then
            ok "$remote (tags)"
        else
            warn "tag push to '$remote' failed"
            FAILED=$((FAILED + 1))
        fi
    fi
done

# -----------------------------------------------------------------------------
# Summary
# -----------------------------------------------------------------------------

printf '\n'
if [ "$DRY_RUN" -eq 1 ]; then
    log "dry run: would push to $PUSHED remote(s)"
    exit 0
fi

if [ "$FAILED" -eq 0 ]; then
    ok "pushed to $PUSHED remote(s)"
    exit 0
fi

die "pushed to $PUSHED remote(s), failed on $FAILED"
