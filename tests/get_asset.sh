#!/bin/bash

# Ensure RPC_URL is set
if [ -z "$RPC_URL" ]; then
    echo "Error: RPC_URL is not set."
    exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

TMP_DIR="$SCRIPT_DIR/tmp-$(date +%Y%m%d-%H%M%S)"
mkdir -p "$TMP_DIR"
if [ ! -d "$TMP_DIR" ]; then
    echo "Failed to create temp directory."
    exit 1
fi
echo "Using temporary directory $TMP_DIR"

ASSET_IDS=(
  "F9Lw3ki3hJ7PF9HQXsBzoY8GyE6sPoEZZdXJBsTTD2rk"
  "J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn"
  "AKo9P7S8FE9NYeAcrtZEpimwQAXJMp8Lrt8p4dMkHkY2"
)

for ASSET_ID in "${ASSET_IDS[@]}";
do
  echo "Processing asset ID: $ASSET_ID"
  CURL_PAYLOAD=$(cat <<-EOF
{
    "jsonrpc": "2.0",
    "id": "my-id",
    "method": "getAsset",
    "params": {
        "id": "$ASSET_ID"
      }
    }
EOF
)

curl --silent -X POST "$RPC_URL" \
  -H "Content-Type: application/json" \
  -d "$CURL_PAYLOAD" | jq . --sort-keys > "$TMP_DIR/curl-get-asset-$ASSET_ID.json"

cargo run --features=cli --quiet -- --url "$RPC_URL" get-asset --asset="$ASSET_ID" \
  | jq . --sort-keys > "$TMP_DIR/rust-get-asset-$ASSET_ID.json"

if ! diff "$TMP_DIR/curl-get-asset-$ASSET_ID.json" "$TMP_DIR/rust-get-asset-$ASSET_ID.json"; then
  echo "Differences found for asset $ASSET_ID. Exiting."
  exit 1
fi
done

#rm -fr $TMP_DIR/
