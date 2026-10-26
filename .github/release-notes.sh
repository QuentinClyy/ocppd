#!/bin/sh
set -eu

range="${1:+$1..}${2:-HEAD}"
subjects=$(git log --no-merges --reverse --format=%s "$range")

entries() {
	printf '%s\n' "$subjects" |
		grep -E "^($1)(\(|!|:)" |
		sed -E \
			-e 's/^[a-z]+\(([^)]*)\)!?: \[([^]]*)\] /- **\1** [\2]: /' \
			-e 's/^[a-z]+\(([^)]*)\)!?: /- **\1**: /' \
			-e 's/^[a-z]+!?: /- /' || true
}

section() {
	lines=$(entries "$2")
	if [ -n "$lines" ]; then
		printf '## %s\n\n%s\n\n' "$1" "$lines"
	fi
}

section "Features" "feat"
section "Fixes" "fix"
section "Performance" "perf"
section "Refactors" "refactor"
section "Docs" "docs"
section "Tests" "test"
section "Build and CI" "build|ci|chore"

other=$(printf '%s\n' "$subjects" | grep -vE '^(feat|fix|perf|refactor|docs|test|build|ci|chore)(\(|!|:)' || true)
if [ -n "$other" ]; then
	printf '## Other\n\n%s\n\n' "$(printf '%s\n' "$other" | sed 's/^/- /')"
fi
