#!/usr/bin/env bash
# Checks the import rules that rustfmt and clippy can't:
# - no `super::super` (reach further up through `crate::`)
# - no `pub(crate)` or `pub(in crate)`: in this binary crate they mean the same as `pub`
# - code outside a feature imports it through `crate::prelude`, never `crate::<feature>::...`
# - a file reaches its own folder through `self::`/`super::`, never `crate::<its folder>::...`
# - a child module is imported with `self::` (`pub use self::child::X`), so rustfmt's
#   `group_imports` doesn't sort it in with external crates
# Warns about prelude entries nothing outside their folder names, since unused glob imports don't
# warn.
set -u
status=0

hits=$(grep -rn 'super::super' src --include='*.rs' | grep -vE '^[^:]+:[0-9]+:\s*//')
if [ -n "$hits" ]; then
  echo "error: use a crate:: path instead of super::super:"
  echo "$hits"
  status=1
fi

hits=$(grep -rnE 'pub\((in )?crate\)' src --include='*.rs' | grep -vE '^[^:]+:[0-9]+:\s*//')
if [ -n "$hits" ]; then
  echo "error: use pub instead of pub(crate); the private top-level mods keep it in the crate:"
  echo "$hits"
  status=1
fi

# A feature is a top-level module that the crate prelude re-exports from.
features=$(awk '/^pub mod prelude \{/ { inside = 1; next } inside && /^\}/ { exit }
  inside && match($0, /super::[a-z_0-9]+/) { print substr($0, RSTART + 7, RLENGTH - 7) }' \
  src/main.rs | sort -u)
if [ -z "$features" ]; then
  echo "error: found no features in the crate prelude in src/main.rs"
  exit 1
fi

for feature in $features; do
  hits=$(grep -rnE "crate::${feature}::" src --include='*.rs' \
    | grep -vE "^src/(${feature}/|${feature}\.rs:|main\.rs:)" \
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

# Every line of code outside prelude blocks, as `path:line`. Comments are dropped and string
# literals (raw and multi-line ones too) are emptied; char literals and lifetimes are told apart.
usages=$(mktemp)
trap 'rm -f "$usages"' EXIT
find src -name '*.rs' | sort | while IFS= read -r file; do
  awk -v file="$file" '
    BEGIN { quote = sprintf("%c", 39) }
    function ident(c) { return c ~ /[A-Za-z0-9_]/ }
    {
      line = $0
      if (state == "" && line ~ /^pub mod prelude \{/) { prelude = 1; next }
      if (prelude) { if (line ~ /^\}/) prelude = 0; next }
      out = ""
      n = length(line)
      for (i = 1; i <= n; i++) {
        c = substr(line, i, 1)
        if (state == "block") {
          two = substr(line, i, 2)
          if (two == "/*") { depth++; i++ }
          else if (two == "*/") { if (--depth == 0) state = ""; i++ }
        } else if (state == "string") {
          if (c == "\\") i++
          else if (c == "\"") state = ""
        } else if (state == "raw") {
          if (substr(line, i, 1 + hashes) == "\"" substr(fence, 1, hashes)) {
            state = ""
            i += hashes
          }
        } else if (substr(line, i, 2) == "//") {
          break
        } else if (substr(line, i, 2) == "/*") {
          state = "block"
          depth = 1
          i++
        } else if (c == "\"") {
          state = "string"
          out = out "\"\""
        } else if (!ident(substr(line, i - 1, 1)) \
          && match(substr(line, i), /^[bc]?r#*"/)) {
          prefix = substr(line, i, RLENGTH)
          hashes = gsub(/#/, "#", prefix)
          fence = "################"
          state = "raw"
          out = out "\"\""
          i += RLENGTH - 1
        } else if (c == quote) {
          if (substr(line, i + 1, 1) == "\\") {
            rest = substr(line, i + 2)
            close_at = index(substr(rest, 2), quote)
            i += close_at + 2
          } else if (substr(line, i + 2, 1) == quote) {
            i += 2
          } else {
            out = out c
          }
        } else {
          out = out c
        }
      }
      if (out ~ /[^ \t]/) print file ":" out
    }
  ' "$file"
done >"$usages"

# Prints `owner name` for each item a prelude block exports. A `mod.rs` prelude exports for its
# folder (`combat`, `combat/status_effects`); a crate prelude line for the `super::` module it
# names.
prelude_entries() {
  local file=$1 feature=$2
  awk '/^pub mod prelude \{/ { inside = 1; next } inside && /^\}/ { exit }
    inside && !/^[ \t]*(#|\/\/)/ { printf "%s ", $0 }' "$file" \
    | tr ';' '\n' \
    | while IFS= read -r statement; do
      case "$statement" in *'*'* | '') continue ;; esac
      path=$(echo "$statement" \
        | sed -E 's/^[[:space:]]*pub(\([^)]*\))?[[:space:]]+use[[:space:]]+//')
      owner=$feature
      if [ -z "$owner" ]; then
        case "$path" in super::*) ;; *) continue ;; esac
        owner=$(echo "$path" | sed -E 's/^super::([a-z_0-9]+).*/\1/')
      fi
      # Splits nested groups (`a::{X, Y}`) into entries and keeps each one's exported name.
      echo "$path" | sed -E 's/^[^{]*\{//; s/\}[[:space:]]*$//' | tr '{},' '\n\n\n' \
        | sed -E 's/.*[[:space:]]as[[:space:]]+//; s/.*:://; s/[[:space:]]//g' | grep -v '^$' \
        | sed "s|^|$owner |"
    done
}

stale=$( {
  prelude_entries src/main.rs ""
  find src -mindepth 2 -name mod.rs | sort | while IFS= read -r file; do
    grep -q '^pub mod prelude {' "$file" || continue
    owner=${file#src/}
    prelude_entries "$file" "${owner%/mod.rs}"
  done
} | sort -u | while read -r owner name; do
  grep -vE "^src/${owner}(/|\.rs:)" "$usages" | grep -qw -- "$name" \
    || echo "  ${owner}: ${name}"
done)
if [ -n "$stale" ]; then
  echo "warning: prelude entries nothing outside their folder uses (drop them, or narrow the item):"
  echo "$stale"
fi

exit $status
