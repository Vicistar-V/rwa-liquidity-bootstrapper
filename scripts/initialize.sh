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

initialize_factory() {
    echo "[1/4] Initializing Pool Factory..."
    soroban contract invoke \
        --id "$POOL_FACTORY_CONTRACT_ID" \
        --source "$SOURCE" \
        --rpc-url "$RPC_URL" \
        --network-passphrase "$NETWORK_PASSPHRASE" \
        -- \
        initialize \
        --admin "$SOURCE"
    echo "  → Pool Factory initialized."
    echo ""
}

register_templates() {
    echo "[2/4] Registering pool templates..."

    soroban contract invoke \
        --id "$POOL_FACTORY_CONTRACT_ID" \
        --source "$SOURCE" \
        --rpc-url "$RPC_URL" \
        --network-passphrase "$NETWORK_PASSPHRASE" \
        -- \
        register_template \
        --pool_type lbp \
        --contract_id "$LBP_TEMPLATE_CONTRACT_ID"

    soroban contract invoke \
        --id "$POOL_FACTORY_CONTRACT_ID" \
        --source "$SOURCE" \
        --rpc-url "$RPC_URL" \
        --network-passphrase "$NETWORK_PASSPHRASE" \
        -- \
        register_template \
        --pool_type bonding \
        --contract_id "$BONDING_TEMPLATE_CONTRACT_ID"

    echo "  → Templates registered."
    echo ""
}

set_admin() {
    echo "[3/4] Setting contract admins..."
    for CONTRACT_ID in "$LBP_TEMPLATE_CONTRACT_ID" "$BONDING_TEMPLATE_CONTRACT_ID" \
                       "$CL_CONTRACT_ID" "$FAIR_LAUNCH_CONTRACT_ID" \
                       "$GRADUATION_ENGINE_CONTRACT_ID" "$TWAP_ORACLE_CONTRACT_ID" \
                       "$LP_REWARDS_CONTRACT_ID" "$COMPLIANCE_BRIDGE_CONTRACT_ID"
    do
        soroban contract invoke \
            --id "$CONTRACT_ID" \
            --source "$SOURCE" \
            --rpc-url "$RPC_URL" \
            --network-passphrase "$NETWORK_PASSPHRASE" \
            -- \
            set_admin \
            --admin "$POOL_FACTORY_CONTRACT_ID"
    done
    echo "  → Admins set."
    echo ""
}

echo "Initialization complete."
echo ""

initialize_factory
register_templates
set_admin
