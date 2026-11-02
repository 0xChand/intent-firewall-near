# Intent Firewall

A testnet vault for AI agents: the owner funds a NEAR smart contract and configures an agent account, per-payment and UTC-day budgets, an allowlist, and an emergency pause. The agent can request native NEAR transfers only through the vault's `pay` method. The contract enforces its rules before creating a transfer. The Cloudflare Workers dashboard reads the actual contract state and can send owner and agent calls through MyNearWallet.

**Live dashboard:** https://near-intent-firewall.rathichand2.workers.dev  
**Demonstration vault:** `intent-firewall-20261102.testnet`  
**Successful restricted-key payment:** https://testnet.nearblocks.io/txns/BXx2u25iLWrtRJq6vPcSqCDqaXDVZq6CmWcgkaDLiMUM

## Try the hosted dashboard

1. Open the dashboard and wait for live vault state. The demo vault is controlled by the developer's testnet account; you may inspect it without a wallet.
2. To make changes, deploy your own vault using the instructions below. Paste its account ID into the dashboard, connect the appropriate testnet wallet and read the state.
3. As owner, set an agent, limits and allowed recipient. Fund the vault with testnet NEAR. As the agent account, submit a payment to an allowlisted account under the limits. A different receiver, excess amount, unauthorized caller or paused vault causes a rejected transaction.
4. Pause the vault in the owner policy section to stop subsequent payments. The owner can withdraw available funds to the owner account.

## Run locally

Install Node.js 20+ and Rust with `wasm32-unknown-unknown`, then:

```sh
npm ci
npm run build
npm test
npm run dev
```

Open the URL printed by Wrangler. To build the contract using near-sdk 5.30's NEAR host configuration:

```sh
rustup target add wasm32-unknown-unknown
cd contract
RUSTFLAGS="--cfg near" cargo build --locked --target wasm32-unknown-unknown --release
cargo test --locked --lib
```

Use the resulting `target/wasm32-unknown-unknown/release/intent_firewall_vault.wasm`. Deploy to a fresh testnet account that you control, initializing atomically with `new` and JSON arguments such as:

```json
{"owner":"YOUR_OWNER.testnet","agent":"YOUR_AGENT.testnet","per_payment":"1000000000000000000000000","daily_limit":"3000000000000000000000000"}
```

Amounts are integer yoctoNEAR strings. Initialization must be called by the contract account itself. Use separate accounts and keys for the contract, owner and agent; keep the contract's full-access deployment key and the owner's key away from the agent. An agent key should be a NEAR function-call access key restricted to the vault account and `pay`, with a bounded gas allowance. The agent account must have enough balance for gas. A function-call key cannot attach a deposit or directly transfer NEAR; it can only call allowed methods on its configured receiver. The contract sends from its own balance after checking the policy. Rotating/revoking an agent key and changing the contract's agent are separate operations.

## Security model and boundaries

Experimental, unaudited, **testnet only**. No mainnet deployment is represented here. The allowlist is controlled by the owner, not a reputation service. The contract checks caller identity, pause state, recipient, per-payment and UTC-calendar-day limits, and reserves funds for pending transfers while protecting the storage stake. Failed transfers still consume the daily budget, conservatively. The balance shown includes the storage stake and should not be interpreted as fully withdrawable. The deployer retains the vault account's full-access key and can upgrade the contract; that key is a root of trust. The demo agent account also has a separately held full-access recovery key; the actual successful agent transaction linked above used a different key scoped to `pay`. This design does not detect prompt injection or provide independent third-party security audits. It restricts the blast radius when instructions cause an agent to request a forbidden payment.

## Verification

Contract unit tests cover unauthorized admin calls, non-allowlisted recipients, payment limit, pause and unauthorized agent. A live testnet transaction used a `pay`-only function-call access key and transferred 0.01 test NEAR to `testnet`; the on-chain callback reported success. Live tests also rejected excess and unlisted payments. Worker policy tests cover the earlier preflight checker, retained as an optional informational API; it is not the vault's enforcement boundary.

Built for the NEAR Hackathon 2026 AI Security & Defense track. Source uses near-sdk, NEAR Wallet Selector, MyNearWallet, Wrangler and esbuild. New code was written during the event window.
