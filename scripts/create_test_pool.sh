#!/usr/bin/env bash
set -euo pipefail

NETWORK="${NETWORK:-testnet}"
RPC_URL="${SOROBAN_RPC_URL:-https://soroban-testnet.stellar.org}"
NETWORK_PASSPHRASE="${NETWORK_PASSPHRASE:-"Test SDF Network ; September 2015"}"
SOURCE="${SOROBAN_SOURCE_ACCOUNT:-admin}"

if [ ! -f .env ]; then
    echo "ERROR: .env file not found. Run scripts/deploy_all.sh first."
    exit 1
fi

source .env

usage() {
    echo "Usage: $0 --rwa-token <address> --amount <amount> [options]"
    echo ""
    echo "Required:"
    echo "  --rwa-token    Address of the RWA token contract"
    echo "  --amount       Amount of RWA tokens to seed (in units, not raw)"
    echo ""
    echo "Optional:"
    echo "  --pool-type         Pool type: lbp (default) | bonding"
    echo "  --weight-start      Starting RWA weight %% (default: 96)"
    echo "  --weight-end        Ending RWA weight %% (default: 50)"
    echo "  --duration-days     Bootstrap duration in days (default: 7)"
    echo "  --purchase-cap      Max USDC per wallet (default: 10000)"
    echo "  --fee-bps           Swap fee in basis points (default: 200)"
    echo "  --start-delay       Hours until pool starts (default: 1)"
    echo "  --curve-type        Bonding curve type: linear | logarithmic (default)"
    echo "  --price-ceiling     Max price for bonding curve"
    echo "  --max-supply        Max token supply for bonding curve"
    echo "  --help              Show this help"
    exit 1
}

RWA_TOKEN=""
AMOUNT=""
POOL_TYPE="lbp"
WEIGHT_START=96
WEIGHT_END=50
DURATION_DAYS=7
PURCHASE_CAP=10000
FEE_BPS=200
START_DELAY=1
CURVE_TYPE="logarithmic"
PRICE_CEILING=""
MAX_SUPPLY=""

while [[ $# -gt 0 ]]; do
    case "$1" in
        --rwa-token) RWA_TOKEN="$2"; shift 2 ;;
        --amount) AMOUNT="$2"; shift 2 ;;
        --pool-type) POOL_TYPE="$2"; shift 2 ;;
        --weight-start) WEIGHT_START="$2"; shift 2 ;;
        --weight-end) WEIGHT_END="$2"; shift 2 ;;
        --duration-days) DURATION_DAYS="$2"; shift 2 ;;
        --purchase-cap) PURCHASE_CAP="$2"; shift 2 ;;
        --fee-bps) FEE_BPS="$2"; shift 2 ;;
        --start-delay) START_DELAY="$2"; shift 2 ;;
        --curve-type) CURVE_TYPE="$2"; shift 2 ;;
        --price-ceiling) PRICE_CEILING="$2"; shift 2 ;;
        --max-supply) MAX_SUPPLY="$2"; shift 2 ;;
        --help) usage ;;
        *) echo "Unknown option: $1"; usage ;;
    esac
done

if [ -z "$RWA_TOKEN" ] || [ -z "$AMOUNT" ]; then
    echo "ERROR: --rwa-token and --amount are required."
    usage
fi

NOW=$(date +%s)
START_TIME=$((NOW + START_DELAY * 3600))
END_TIME=$((START_TIME + DURATION_DAYS * 86400))

WEIGHT_START_SCALED=$((WEIGHT_START * 100000))
WEIGHT_END_SCALED=$((WEIGHT_END * 100000))
AMOUNT_SCALED="${AMOUNT}_0000000"
PURCHASE_CAP_SCALED="${PURCHASE_CAP}_0000000"

echo "=== Creating $POOL_TYPE Pool ==="
echo "  RWA Token:      $RWA_TOKEN"
echo "  Amount:         $AMOUNT tokens"
echo "  Duration:       $DURATION_DAYS days"
echo "  Start:          $(date -d @"$START_TIME")"
echo "  End:            $(date -d @"$END_TIME")"
echo ""

if [ "$POOL_TYPE" = "lbp" ]; then
    soroban contract invoke \
        --id "$POOL_FACTORY_CONTRACT_ID" \
        --source "$SOURCE" \
        --rpc-url "$RPC_URL" \
        --network-passphrase "$NETWORK_PASSPHRASE" \
        -- \
        create_lbp_pool \
        --issuer "$SOURCE" \
        --rwa_token "$RWA_TOKEN" \
        --rwa_amount "$AMOUNT_SCALED" \
        --weight_rwa_start "$WEIGHT_START_SCALED" \
        --weight_rwa_end "$WEIGHT_END_SCALED" \
        --start_time "$START_TIME" \
        --end_time "$END_TIME" \
        --swap_fee_bps "$FEE_BPS" \
        --purchase_cap_per_wallet "$PURCHASE_CAP_SCALED" \
        --kyc_required false
elif [ "$POOL_TYPE" = "bonding" ]; then
    soroban contract invoke \
        --id "$POOL_FACTORY_CONTRACT_ID" \
        --source "$SOURCE" \
        --rpc-url "$RPC_URL" \
        --network-passphrase "$NETWORK_PASSPHRASE" \
        -- \
        create_bonding_pool \
        --issuer "$SOURCE" \
        --rwa_token "$RWA_TOKEN" \
        --curve_type "$CURVE_TYPE" \
        --coefficient_a 10000000 \
        --coefficient_b 10000000 \
        --max_supply "${MAX_SUPPLY:-${AMOUNT}}_0000000" \
        --price_ceiling "${PRICE_CEILING:-20000000}" \
        --purchase_cap_per_wallet "$PURCHASE_CAP_SCALED" \
        --kyc_required false
else
    echo "ERROR: Unknown pool type: $POOL_TYPE"
    exit 1
fi
