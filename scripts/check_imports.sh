#!/usr/bin/env bash
# Checks the import rules that rustfmt and clippy can't:
# - no `super::super` (reach further up through `crate::`)
# - code outside a feature imports it through `crate::prelude`, never `crate::<feature>::...`
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

exit $status
