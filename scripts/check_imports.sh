#!/usr/bin/env bash
# Checks the import rules that rustfmt and clippy can't:
# - no `super::super` (reach further up through `crate::`)
# - code outside a feature imports it through `crate::prelude`, never `crate::<feature>::...`
# - a file reaches its own folder through `self::`/`super::`, never `crate::<its folder>::...`
# - a child module is imported with `self::` (`pub use self::child::X`), so rustfmt's
#   `group_imports` doesn't sort it in with external crates
# Warns about prelude entries that no other feature names, since unused glob imports don't warn.
set -u
status=0

hits=$(grep -rn 'super::super' src --include='*.rs' | grep -vE '^[^:]+:[0-9]+:\s*//')
if [ -n "$hits" ]; then
  echo "error: use a crate:: path instead of super::super:"
  echo "$hits"
  status=1
fi

# A feature is a top-level module that the crate prelude re-exports from.
features=$(awk '/^pub mod prelude \{/ { inside = 1; next } inside && /^\}/ { exit }
  inside && match($0, /super::[a-z_0-9]+/) { print substr($0, RSTART + 7, RLENGTH - 7) }' \
  src/main.rs | sort -u)

for feature in $features; do
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

# Every line of code outside comments and prelude blocks, as `path:line`.
usages=$(mktemp)
trap 'rm -f "$usages"' EXIT
find src -name '*.rs' | sort | while IFS= read -r file; do
  awk -v file="$file" '
    /^pub mod prelude \{/ { inside = 1; next }
    inside { if (/^\}/) inside = 0; next }
    /^[ \t]*\/\// { next }
    { print file ":" $0 }
  ' "$file"
done >"$usages"

# Prints `feature name` for each item a prelude block exports. A feature's `mod.rs` prelude
# exports for that feature; a crate prelude line exports for the module it names.
prelude_entries() {
  local file=$1 feature=$2
  awk '/^pub mod prelude \{/ { inside = 1; next } inside && /^\}/ { exit }
    inside && !/^[ \t]*#/ { printf "%s ", $0 }' "$file" \
    | tr ';' '\n' \
    | while IFS= read -r statement; do
      case "$statement" in *'*'* | '') continue ;; esac
      path=$(echo "$statement" | sed -E 's/^[[:space:]]*pub(\([^)]*\))?[[:space:]]+use[[:space:]]+//')
      owner=$feature
      if [ -z "$owner" ]; then
        owner=$(echo "$path" | sed -E 's/^super::([a-z_0-9]+).*/\1/')
      fi
      case "$path" in
        *'{'*) names=$(echo "$path" | sed -E 's/^[^{]*\{//; s/\}.*$//' | tr ',' '\n') ;;
        *) names=${path##*::} ;;
      esac
      echo "$names" | sed -E 's/.* as //; s/[[:space:]]//g' | grep -v '^$' \
        | sed "s/^/$owner /"
    done
}

stale=$( {
  prelude_entries src/main.rs ""
  for file in src/*/mod.rs; do
    grep -q '^pub mod prelude {' "$file" || continue
    feature=${file#src/}
    prelude_entries "$file" "${feature%/mod.rs}"
  done
} | sort -u | while read -r feature name; do
  grep -vE "^src/${feature}(/|\.rs:)" "$usages" | grep -qw -- "$name" \
    || echo "  ${feature}: ${name}"
done)
if [ -n "$stale" ]; then
  echo "warning: prelude entries no other feature uses (drop them, or narrow the item):"
  echo "$stale"
fi

exit $status
