#!/usr/bin/env bash

set -eo pipefail

TARGET="thumbv7m-none-eabi"

packages=$(cargo metadata --format-version 1 --no-deps | jq -r '
  .packages[]

  | select(.manifest_path | contains("/core/"))
  | select([.targets[].kind[]] | contains(["proc-macro"]) | not)
  | .name
')

if [ -z "$packages" ]; then
    exit 0
fi

if ! rustup target list | grep -q "$TARGET (installed)"; then
    echo "📥 Install $TARGET..."
    rustup target add "$TARGET"
fi

for pkg in $packages; do
    echo "--------------------------------------------------"
    echo "🔨 Run cargo check for [$pkg] ($TARGET)..."
    echo "--------------------------------------------------"

    cargo check -p "$pkg" --target "$TARGET" || {
        exit 1
    }
done

echo "--------------------------------------------------"
echo "✅ Success"
