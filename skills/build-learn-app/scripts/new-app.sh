#!/bin/bash
# Creates a learning-app workspace from templates/starter.
#
# Usage: new-app.sh <project-dir> <slug> "<Display Name>" [bundle-prefix]
#   project-dir    may already hold .git, .gitignore, docs/, research/, notes/, target/
#                  (the research and grilling log of phases 0–3); those files
#                  are never overwritten
#   slug           lowercase letters and digits, starts with a letter (binary, crates, env prefix)
#   Display Name   window title, bundle name, data folder name (letters, digits, spaces, - . ,)
#   bundle-prefix  reverse DNS, default com.example  → bundle id <prefix>.<slug>
#
# Replaces learnapp → slug, Learnapp → Display Name, LEARNAPP → SLUG, com.example → prefix
# in every copied text file and path. Binary files (fonts, icon) are copied as they are.
# Does not commit; the lead does that after SPEC.md is written.
set -euo pipefail
SKILL="$(cd "$(dirname "$0")/.." && pwd)"
SRC="$SKILL/templates/starter"
DEST="${1:?project dir required}"
SLUG="${2:?slug required}"
NAME="${3:?display name required}"
PREFIX="${4:-com.example}"

[[ "$SLUG" =~ ^[a-z][a-z0-9]{1,30}$ ]] || { echo "slug must match ^[a-z][a-z0-9]{1,30}\$ (got '$SLUG')"; exit 2; }
[[ "$PREFIX" =~ ^[a-z][a-z0-9-]*(\.[a-z0-9-]+)+$ ]] || { echo "bundle prefix must be reverse DNS like com.yourname (got '$PREFIX')"; exit 2; }
# The name goes into Info.plist (XML), shell commands and a folder name.
[[ "$NAME" =~ ^[A-Za-z0-9][A-Za-z0-9\ .,-]{0,39}$ ]] || { echo "display name: letters, digits, spaces and - . , only, at most 40 characters (got '$NAME')"; exit 2; }

mkdir -p "$DEST"
DEST="$(cd "$DEST" && pwd)"
OTHER="$(ls -A "$DEST" | grep -v -x -e .git -e .gitignore -e docs -e research -e notes -e target -e .DS_Store || true)"
[ -z "$OTHER" ] || { echo "project dir holds more than .git, .gitignore, docs/, research/, notes/, target/: $DEST"; echo "$OTHER"; exit 2; }

# Files that were there before (the phases 0–3 work): never overwritten, never rewritten.
PRE="$(cd "$DEST" && find . -type f -not -path './.git/*' -not -path './target/*' | sort)"
is_pre() { [ -n "$PRE" ] && grep -Fxq "./$1" <<<"$PRE"; }

rsync -a --ignore-existing --exclude target --exclude dist --exclude evidence --exclude .DS_Store "$SRC/" "$DEST/"

# Merge the starter's ignore rules into an existing .gitignore.
if is_pre .gitignore; then
  while IFS= read -r line; do
    if [ -n "$line" ] && ! grep -Fxq -- "$line" "$DEST/.gitignore"; then
      printf '%s\n' "$line" >>"$DEST/.gitignore"
    fi
  done <"$SRC/.gitignore"
fi

UPPER="$(echo "$SLUG" | tr '[:lower:]' '[:upper:]')"
export SLUG NAME UPPER PREFIX

# Text files from the starter: replace the placeholders.
while IFS= read -r -d '' f; do
  rel="${f#"$DEST"/}"
  is_pre "$rel" && continue
  if grep -Iq . "$f" 2>/dev/null; then
    perl -pi -e 's/com\.example/$ENV{PREFIX}/g; s/LEARNAPP/$ENV{UPPER}/g; s/Learnapp/$ENV{NAME}/g; s/learnapp/$ENV{SLUG}/g' "$f"
  fi
done < <(find "$DEST" -type f -not -path "$DEST/.git/*" -not -path "$DEST/target/*" -print0)

# Paths: deepest first so parents are renamed after their children.
while IFS= read -r p; do
  base="$(basename "$p")"
  new="${base//learnapp/$SLUG}"
  [ "$new" != "$base" ] && mv "$p" "$(dirname "$p")/$new"
done < <(find "$DEST" -depth -name '*learnapp*' -not -path "$DEST/.git/*" -not -path "$DEST/target/*")

chmod +x "$DEST"/tools/*.sh "$DEST"/tools/*.py "$DEST"/.claude/skills/verify-"$SLUG"/scripts/*.sh

LEFT=""
while IFS= read -r f; do
  rel="${f#"$DEST"/}"
  is_pre "$rel" || LEFT="$LEFT$f"$'\n'
done < <(grep -rIl -e learnapp -e Learnapp -e LEARNAPP "$DEST" --exclude-dir=.git --exclude-dir=target 2>/dev/null || true)
if [ -n "$LEFT" ]; then
  echo "placeholders left in:"; printf '%s' "$LEFT"; exit 1
fi

# New names have new lengths: let rustfmt re-wrap so `cargo fmt --check` passes.
if command -v cargo >/dev/null 2>&1; then
  (cd "$DEST" && cargo fmt --all) || { echo "cargo fmt failed in $DEST"; exit 1; }
fi

cat <<EOF
Created $NAME in $DEST
  binary $SLUG, bundle id $PREFIX.$SLUG, data folder ~/Library/Application Support/$NAME
  verify skill: .claude/skills/verify-$SLUG/
  kept as they were: $(printf '%s\n' "$PRE" | grep -c . || true) existing file(s)
  next: cd "$DEST" && .claude/skills/verify-$SLUG/scripts/gate.sh
EOF
