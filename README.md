Overview:

Automated trading (MEV) bots interact with decentralized pools using simple profit rules. These bots are programmed to act whenever their calculations show a profit, ignoring small costs.

System Flaw:

The flaw: Bots trust their simulations and ignore minor, hidden costs. The system they operate in is deterministic and rule-based. This makes them vulnerable to contracts that take a small fee without detection.

Exploitation:

Bob builds a contract that copies normal pool behavior but inserts a small, hidden fee. He uses a watcher to monitor pools for bot activity and activates the trap where it is most effective.

Proof of Mechanism:

- Bots always act when profit > cost.
- The trap fee is set below their detection threshold.
- Contract logic and gas usage disguise the fee from simulations.
- The watcher can target any pool and adapt as bot activity moves.

Why Systems Are Exploitable:

- Bots focus on maximizing gains, not monitoring tiny losses.
- The decentralized pool system does not prevent hidden fees if contracts look standard.
- Automated strategies repeat mistakes if the system rewards speed and volume over scrutiny.

Outcomes:

- Fees are collected automatically from bot trades.
- The process can move to new pools and chains as needed.
- The exploit persists as long as bots use deterministic logic and pools are liquid.

Human Analogy:

People chasing small profits often ignore minor costs that add up over time. Bots behave the same way, making them vulnerable to well-designed traps.

Usage:

1. Deploy the trap contract.
2. Run the watcher, pointing it at an active pool.
3. Collect fees from bot activity.
4. Update targets as needed.

Summary:
This system demonstrates how rule-based automation can be exploited by introducing hidden, undetectable costs. The exploit works across pools and chains, adapting as bot activity changes.
