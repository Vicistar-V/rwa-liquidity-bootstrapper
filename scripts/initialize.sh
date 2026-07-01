#!/usr/bin/env bash
set -euo pipefail

NETWORK="${1:-testnet}"
RPC_URL="${SOROBAN_RPC_URL:-https://soroban-testnet.stellar.org}"
NETWORK_PASSPHRASE="${NETWORK_PASSPHRASE:-"Test SDF Network ; September 2015"}"
SOURCE="${SOROBAN_SOURCE_ACCOUNT:-admin}"

if [ ! -f .env ]; then
    echo "ERROR: .env file not found. Run scripts/deploy_all.sh first."
    exit 1
fi

source .env

echo "=== RWA-LBP Initialize ==="
echo ""

init_factory() {
    echo "[1/2] Initializing Pool Factory..."
    soroban contract invoke \
        --id "$POOL_FACTORY_CONTRACT_ID" \
        --source "$SOURCE" \
        --rpc-url "$RPC_URL" \
        --network-passphrase "$NETWORK_PASSPHRASE" \
        -- \
        init \
        --admin "$SOURCE" \
        --lbp_contract "$LBP_TEMPLATE_CONTRACT_ID" \
        --bonding_contract "$BONDING_TEMPLATE_CONTRACT_ID" \
        --cl_contract "$CL_CONTRACT_ID" \
        --fair_launch_contract "$FAIR_LAUNCH_CONTRACT_ID" \
        --oracle_contract "$TWAP_ORACLE_CONTRACT_ID"
    echo "  → Pool Factory initialized."
    echo ""
}

init_graduation_engine() {
    echo "[2/2] Initializing Graduation Engine contract addresses..."
    soroban contract invoke \
        --id "$GRADUATION_ENGINE_CONTRACT_ID" \
        --source "$SOURCE" \
        --rpc-url "$RPC_URL" \
        --network-passphrase "$NETWORK_PASSPHRASE" \
        -- \
        set_factory \
        --factory "$POOL_FACTORY_CONTRACT_ID"

    soroban contract invoke \
        --id "$GRADUATION_ENGINE_CONTRACT_ID" \
        --source "$SOURCE" \
        --rpc-url "$RPC_URL" \
        --network-passphrase "$NETWORK_PASSPHRASE" \
        -- \
        set_fair_launch \
        --fair_launch "$FAIR_LAUNCH_CONTRACT_ID"

    echo "  → Graduation Engine configured."
    echo ""
}

echo "Initialization complete."
echo ""

init_factory
init_graduation_engine
