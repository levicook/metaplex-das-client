#!/bin/bash

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

OWNER_ADDRESSES=(
  "86xCnPeV69n6t3DnyGvkKobf9FdN2H9oiVDdaMpo2MMY"
  "JUskoxS2PTiaBpxfGaAPgf3cUNhdeYFGMKdL6mZKKfR"
)

for OWNER_ADDRESS in "${OWNER_ADDRESSES[@]}";
do
  echo "Processing owner address: $OWNER_ADDRESS"
  CURL_PAYLOAD=$(cat <<-EOF
{
    "jsonrpc": "2.0",
    "id": "my-id",
    "method": "getAssetsByOwner",
    "params": {
        "ownerAddress": "$OWNER_ADDRESS",
        "page": 1,
        "limit": 1000,
        "displayOptions": {
          "showFungible": true
        }
      }
    }
EOF
)

curl --silent -X POST "$RPC_URL" \
  -H "Content-Type: application/json" \
  -d "$CURL_PAYLOAD" | jq . --sort-keys > "$TMP_DIR/curl-get-assets-by-owner-$OWNER_ADDRESS.json"

cargo run --features=cli --quiet -- --url "$RPC_URL" get-assets-by-owner --owner="$OWNER_ADDRESS" \
  | jq . --sort-keys > "$TMP_DIR/rust-get-assets-by-owner-$OWNER_ADDRESS.json"

if ! diff "$TMP_DIR/curl-get-assets-by-owner-$OWNER_ADDRESS.json" "$TMP_DIR/rust-get-assets-by-owner-$OWNER_ADDRESS.json"; then
  echo "Differences found for assets $OWNER_ADDRESS. Exiting."
  exit 1
fi
done
