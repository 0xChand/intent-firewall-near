# Intent Firewall

An explainable pre-signing checkpoint for NEAR agent payment proposals, built for NEAR Hackathon 2026, AI Security & Defense. A Cloudflare Worker checks whether a receiver exists using NEAR RPC, then evaluates an explicit per-action budget and caller-supplied trust, blocklist and injection flags. It returns allow, warn, or block with reasons. A small agent wrapper gates an injected execute callback on the policy outcome and human approval.

## Try it

1. Install Node.js 20+ and run `npm install` then `npm run dev`.
2. Open the local URL. The default `near` account, 2 NEAR amount, 5 NEAR limit and trusted flag should return ALLOW when mainnet RPC is available.
3. Uncheck trusted and evaluate again: see HUMAN REVIEW. Click Load attack scenario: see BLOCK and the reasons.
4. Run `npm test` to exercise policy decisions, RPC parsing and the guard boundary.

POST `/api/check` accepts `{ "receiver": "near", "amount": 2, "limit": 5, "network": "mainnet", "trusted": true, "blocked": false, "injection": false }`. It returns a decision, reasons and RPC status. `src/guard.js` demonstrates calling policy before an action and requires explicit approval for warn. Integrators must protect that boundary and independently authenticate policy and approval requests.

## Scope and security

This is a prototype, not a deployed wallet, smart contract, or transaction signer. No assets are held or moved. Trust, blocklist, injection flags and limits are supplied by the client; the demo does not detect injection automatically, verify historical reputation, or provide tamper-resistant storage. It is not safe to rely on the public endpoint for real funds. RPC errors produce warn, and the wrapper stops unless a human explicitly approves. Account existence is a point-in-time read, not proof of safety. Amounts use JavaScript numbers and are illustrative, not suitable for token accounting.

For production, enforce a policy in an independently controlled contract, use exact integer token units, authenticated approvals and durable rules, and audit the integration. NEAR function-call access keys can be limited to a receiver contract and method names, but cannot make native NEAR transfers or attach deposits; they do not protect against unsafe behavior inside the allowed contract. A full-access key is not used here. No testnet or mainnet contract is deployed.

Built from scratch during the hackathon window. Runtime is Cloudflare Workers; local tooling uses Wrangler. No NEAR AI model or third-party blocklist is claimed.
