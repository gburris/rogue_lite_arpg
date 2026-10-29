#!/usr/bin/env bash
# Checks the import rules that rustfmt and clippy can't:
# - no `super::super` (reach further up through `crate::`)
# - code outside a feature imports it through `crate::prelude`, never `crate::<feature>::...`
# - a file reaches its own folder through `self::`/`super::`, never `crate::<its folder>::...`
# - a child module is imported with `self::` (`pub use self::child::X`), so rustfmt's
#   `group_imports` doesn't sort it in with external crates
set -u
status=0

hits=$(grep -rn 'super::super' src --include='*.rs' | grep -vE '^[^:]+:[0-9]+:\s*//')
if [ -n "$hits" ]; then
  echo "error: use a crate:: path instead of super::super:"
  echo "$hits"
  status=1
fi

for feature in animation character combat configuration items menu world; do
  hits=$(grep -rnE "crate::${feature}::" src --include='*.rs' \
    --exclude-dir="$feature" --exclude="${feature}.rs" --exclude=main.rs \
    | grep -vE '^[^:]+:[0-9]+:\s*//')
  if [ -n "$hits" ]; then
    echo "error: import ${feature} items through crate::prelude, not crate::${feature}::..."
    echo "$hits"
    status=1
  fi
done

hits=$(find src -name '*.rs' | sort | while IFS= read -r file; do
  dir=${file#src/}
  case "$dir" in */*) dir=${dir%/*} ;; *) continue ;; esac
  grep -nF "crate::${dir//\//::}::" "$file" | grep -vE '^[0-9]+:\s*//' | sed "s|^|$file:|"
done)
if [ -n "$hits" ]; then
  echo "error: reach your own folder through self:: or super::, not crate::<folder>::..."
  echo "$hits"
  status=1
fi

hits=$(find src -name '*.rs' | sort | while IFS= read -r file; do
  awk -v file="$file" '
    FNR == NR {
      if (match($0, /^[ \t]*(pub(\([^)]*\))?[ \t]+)?mod[ \t]+[a-z_0-9]+[ \t]*;/)) {
        name = $0
        sub(/^.*mod[ \t]+/, "", name)
        sub(/[ \t]*;.*$/, "", name)
        mods[name] = 1
      }
      next
    }
    match($0, /^[ \t]*(pub(\([^)]*\))?[ \t]+)?use[ \t]+[a-z_0-9]+::/) {
      name = $0
      sub(/^.*use[ \t]+/, "", name)
      sub(/::.*$/, "", name)
      if (name in mods) print file ":" FNR ":" $0
    }
  ' "$file" "$file"
done)
if [ -n "$hits" ]; then
  echo "error: import a child module with self:: (pub use self::child::X):"
  echo "$hits"
  status=1
fi

exit $status
