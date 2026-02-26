# solana-defi

DeFi protocol implementations on Solana. Built with Anchor 0.32.1 and tested with LiteSVM 0.7.

## Protocols

### kpool — Constant Product AMM

Uniswap V2-style automated market maker. Liquidity providers deposit token pairs, traders swap against the pool, and fees accrue to LPs through the constant product invariant (`x * y = k`).

**Instructions**: `initialize_pool`, `add_liquidity`, `swap`, `remove_liquidity`

```bash
cd kpool && anchor build
cd kpool/tests-litesvm && cargo test
```

### klend — Lending/Borrowing

Aave V2/Compound V2-style lending protocol. Users deposit collateral, borrow against it, earn interest from borrowers, and face liquidation if undercollateralized.

- Kinked interest rate model (base + slope1/slope2 around optimal utilization)
- Compound cToken-style exchange rates with virtual shares for inflation attack defense
- Health factor checks on borrow/withdraw, liquidation with 50% close factor and 5% bonus
- Mock oracle for testing (swappable for Pyth/Switchboard in production)

**Instructions**: `init_market`, `init_mock_oracle`, `update_mock_oracle`, `init_reserve`, `refresh_reserve`, `deposit`, `withdraw`, `borrow`, `repay`, `liquidate`

```bash
cd klend && anchor build
cd klend/tests-litesvm && cargo test
```

## Research

The `research/` directory contains 9 deep-dive documents (~74,000 words) covering DeFi fundamentals, AMMs, lending, yield sources, stablecoins, the Solana ecosystem, key papers and math, protocol design patterns, and advanced strategies.

## Stack

- **Anchor** 0.32.1
- **LiteSVM** 0.7 (standalone test crates, excluded from workspace)
- **Solana SDK** 2.x / SPL Token 7.x
