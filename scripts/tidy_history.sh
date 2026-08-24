#!/usr/bin/env bash
set -euo pipefail

repo_root=$(git rev-parse --show-toplevel)
cd "$repo_root"

git_dir=$(git rev-parse --git-dir)
for state in \
  "$git_dir/MERGE_HEAD" \
  "$git_dir/CHERRY_PICK_HEAD" \
  "$git_dir/REVERT_HEAD" \
  "$(git rev-parse --git-path rebase-merge)" \
  "$(git rev-parse --git-path rebase-apply)"; do
  if [[ -e "$state" ]]; then
    echo "Cannot tidy history while an operation is in progress: $state" >&2
    exit 2
  fi
done

if [[ -n "$(git status --porcelain --untracked-files=all)" ]]; then
  echo "Cannot tidy history with a dirty working tree" >&2
  git status --short >&2
  exit 2
fi

if ! upstream=$(git rev-parse --abbrev-ref --symbolic-full-name '@{upstream}' 2>/dev/null); then
  echo "Cannot tidy history: current branch has no upstream" >&2
  exit 2
fi

if ! git merge-base --is-ancestor "$upstream" HEAD; then
  echo "Cannot tidy history: upstream is not an ancestor of HEAD" >&2
  exit 2
fi

mapfile -t commits < <(git log --reverse --format='%H%x09%s' "$upstream..HEAD")
if ((${#commits[@]} < 2)); then
  echo "No adjacent metadata-only commits to tidy."
  exit 0
fi

tmpdir=$(mktemp -d)
trap 'rm -rf "$tmpdir"' EXIT

todo="$tmpdir/todo"
: > "$todo"

metadata_only() {
  local commit=$1 path
  while IFS= read -r path; do
    [[ "$path" == .tandem/* ]] || return 1
  done < <(git diff-tree --no-commit-id --name-only -r --root "$commit")
  [[ -n "$(git diff-tree --no-commit-id --name-only -r --root "$commit")" ]]
}

squash_count=0
previous_metadata=false
for entry in "${commits[@]}"; do
  commit=${entry%%$'\t'*}
  subject=${entry#*$'\t'}
  if metadata_only "$commit"; then
    if $previous_metadata; then
      printf 'squash %s %s\n' "$commit" "$subject" >> "$todo"
      ((squash_count += 1))
    else
      printf 'pick %s %s\n' "$commit" "$subject" >> "$todo"
    fi
    previous_metadata=true
  else
    printf 'pick %s %s\n' "$commit" "$subject" >> "$todo"
    previous_metadata=false
  fi
done

if ((squash_count == 0)); then
  echo "No adjacent metadata-only commits to tidy."
  exit 0
fi

sequence_editor="$tmpdir/sequence-editor"
cat > "$sequence_editor" <<EOF
#!/usr/bin/env bash
cp "$todo" "\$1"
EOF
chmod +x "$sequence_editor"

message_editor="$tmpdir/message-editor"
cat > "$message_editor" <<EOF
#!/usr/bin/env bash
cat > "\$1" <<'MESSAGE'
coord(tandem): consolidate adjacent metadata checkpoints

The adjacent metadata-only checkpoints were combined before push.
MESSAGE
EOF
chmod +x "$message_editor"

GIT_SEQUENCE_EDITOR="$sequence_editor" \
GIT_EDITOR="$message_editor" \
  git rebase -i "$upstream"

echo "Tidied $squash_count metadata-only commit(s) into adjacent checkpoint run(s)."
