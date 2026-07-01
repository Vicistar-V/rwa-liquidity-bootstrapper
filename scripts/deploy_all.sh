#!/usr/bin/env bash
set -euo pipefail

NETWORK="${1:-testnet}"
RPC_URL="${SOROBAN_RPC_URL:-https://soroban-testnet.stellar.org}"
NETWORK_PASSPHRASE="${NETWORK_PASSPHRASE:-"Test SDF Network ; September 2015"}"
SOURCE="${SOROBAN_SOURCE_ACCOUNT:-admin}"

WASM_DIR="target/wasm32-unknown-unknown/release"

echo "=== RWA-LBP Deploy All ==="
echo "Network: $NETWORK"
echo "RPC URL: $RPC_URL"
echo ""

build_contracts() {
    echo "[1/3] Building contracts..."
    cargo build --target wasm32-unknown-unknown --release
    echo "Build complete."
    echo ""
}

deploy_contract() {
    local wasm_name="$1"
    local wasm_path="$WASM_DIR/${wasm_name}.wasm"

    if [ ! -f "$wasm_path" ]; then
        echo "ERROR: Wasm file not found at $wasm_path"
        exit 1
    fi

    echo "  Deploying $wasm_name..."
    local contract_id
    contract_id=$(soroban contract deploy \
        --wasm "$wasm_path" \
        --source "$SOURCE" \
        --rpc-url "$RPC_URL" \
        --network-passphrase "$NETWORK_PASSPHRASE" \
        2>/dev/null)

    echo "  → $contract_id"
    echo "$contract_id"
}

deploy_all() {
    echo "[2/3] Deploying contracts..."

    FACTORY_ID=$(deploy_contract "contract_factory")
    LBP_ID=$(deploy_contract "contract_lbp")
    BONDING_ID=$(deploy_contract "contract_bonding")
    CL_ID=$(deploy_contract "contract_cl")
    FAIRLAUNCH_ID=$(deploy_contract "contract_fairlaunch")
    GRADUATION_ID=$(deploy_contract "contract_graduation")
    ORACLE_ID=$(deploy_contract "contract_oracle")
    REWARDS_ID=$(deploy_contract "contract_rewards")
    COMPLIANCE_ID=$(deploy_contract "contract_compliance")

    echo ""
    echo "All contracts deployed."
    echo ""
}

write_env() {
    echo "[3/3] Writing .env file..."

    cat > .env <<EOF
# Stellar Network
STELLAR_NETWORK=$NETWORK
SOROBAN_RPC_URL=$RPC_URL
NETWORK_PASSPHRASE=$NETWORK_PASSPHRASE

# Contract IDs
POOL_FACTORY_CONTRACT_ID=$FACTORY_ID
LBP_TEMPLATE_CONTRACT_ID=$LBP_ID
BONDING_TEMPLATE_CONTRACT_ID=$BONDING_ID
CL_CONTRACT_ID=$CL_ID
FAIR_LAUNCH_CONTRACT_ID=$FAIRLAUNCH_ID
GRADUATION_ENGINE_CONTRACT_ID=$GRADUATION_ID
TWAP_ORACLE_CONTRACT_ID=$ORACLE_ID
LP_REWARDS_CONTRACT_ID=$REWARDS_ID
COMPLIANCE_BRIDGE_CONTRACT_ID=$COMPLIANCE_ID
EOF

    echo ".env file written."
    echo ""
}

verify_deployment() {
    echo ""
    echo "=== Deployment Summary ==="
    printf "  %-30s %s\n" "Pool Factory:" "$FACTORY_ID"
    printf "  %-30s %s\n" "LBP Template:" "$LBP_ID"
    printf "  %-30s %s\n" "Bonding Curve:" "$BONDING_ID"
    printf "  %-30s %s\n" "Concentrated Liquidity:" "$CL_ID"
    printf "  %-30s %s\n" "Fair Launch Controller:" "$FAIRLAUNCH_ID"
    printf "  %-30s %s\n" "Graduation Engine:" "$GRADUATION_ID"
    printf "  %-30s %s\n" "TWAP Oracle:" "$ORACLE_ID"
    printf "  %-30s %s\n" "LP Rewards:" "$REWARDS_ID"
    printf "  %-30s %s\n" "Compliance Bridge:" "$COMPLIANCE_ID"
    echo ""
    echo "Deployment complete."
}

build_contracts
deploy_all
write_env
verify_deployment
