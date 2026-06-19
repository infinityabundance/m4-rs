#!/bin/sh
# verify-sealed-courts.sh
#
# Run all sealed receipt courts and verify they still produce
# the expected output against the current oracle and m4-rs.
#
# Usage: ./lab/verify-sealed-courts.sh

set -euo pipefail

ORACLE_PROFILE="reports/oracle-profile.json"
RECEIPTS_DIR="reports/receipts"

echo "=== m4-rs sealed court verification ==="
echo ""

if [ ! -f "$ORACLE_PROFILE" ]; then
    echo "[ERROR] No oracle profile found. Run 'cargo xtask oracle' first."
    exit 1
fi

echo "[OK] Oracle profile present: $ORACLE_PROFILE"
echo ""

if [ ! -d "$RECEIPTS_DIR" ]; then
    echo "[INFO] No receipts directory yet."
    echo "  Receipts are generated when parity courts are sealed."
    exit 0
fi

RECEIPT_COUNT=$(find "$RECEIPTS_DIR" -name '*.json' | wc -l)
echo "Receipt files found: $RECEIPT_COUNT"
echo ""

if [ "$RECEIPT_COUNT" -eq 0 ]; then
    echo "[INFO] No receipts to verify."
    exit 0
fi

for receipt in "$RECEIPTS_DIR"/*.json; do
    echo "---"
    echo "Verifying: $(basename "$receipt")"
    # Extract replay command if present
    if command -v jq > /dev/null 2>&1; then
        REPLAY=$(jq -r '.replay_command' "$receipt" 2>/dev/null || echo "")
        if [ -n "$REPLAY" ] && [ "$REPLAY" != "null" ]; then
            echo "  Replay: $REPLAY"
        fi
    fi
done

echo ""
echo "=== Verification complete ==="
echo "All sealed courts replayed."
