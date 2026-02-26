# kvault — Yield Vault

ERC-4626/Yearn V3-style yield vault on Solana. Users deposit an underlying token (e.g. USDC) and receive fungible share tokens. An admin allocates idle funds into klend (lending protocol) via CPI to earn interest. Yield is harvested periodically and performance + management fees are extracted through dilutive share minting.

## Instructions

| Instruction | Description |
|---|---|
| `init_vault` | Create vault PDA, share mint, vault ATA. Fund authority PDA with 0.1 SOL for klend obligation rent. Set fee rates and deposit cap. |
| `deposit` | User deposits underlying tokens, receives shares. Blocked when halted. Respects deposit cap. |
| `withdraw` | User burns shares, receives underlying from idle balance. Works even when halted (users must always be able to exit). |
| `allocate` | Admin deploys idle tokens to klend via CPI (`klend::deposit`). |
| `deallocate` | Admin pulls tokens from klend via CPI (`klend::withdraw`). Uses `reload()` to measure actual received amount. |
| `harvest` | Read klend reserve + obligation state to compute current invested value. Calculate yield, mint dilutive fee shares to fee recipient. Update cached `total_invested`. |
| `set_halt` | Admin toggles emergency halt. Blocks deposits but not withdrawals. |

## Account Architecture

```
Vault PDA            ["vault", underlying_mint]                Vault config and state
Vault Authority      ["vault_authority", vault]                Signs token transfers + klend CPI
Share Mint           ["share_mint", vault]                     Fungible SPL token (6 decimals)
Vault Token Account  ATA(vault_authority, underlying_mint)     Holds idle (unallocated) tokens
```

The vault authority also owns the klend obligation, enabling CPI deposits and withdrawals.

## Share Math (ERC-4626)

Virtual offset of 1 prevents inflation/donation attacks:

```
total_assets = idle_balance + total_invested

Deposit:
  shares = amount * (supply + 1) / (total_assets + 1)         // rounds down

Withdraw:
  amount = shares * (total_assets + 1) / (supply + 1)         // rounds down
```

## Fee Model (Yearn V3)

Fees are taken by minting new share tokens to the fee recipient, diluting existing holders by exactly the fee amount. No token transfers needed.

```
Performance fee:  yield * performance_fee_bps / 10000          (on harvest, only on positive yield)
Management fee:   total_assets * management_fee_bps * elapsed / (10000 * seconds_per_year)

Fee shares minted:
  fee_shares = fee_underlying * (supply + 1) / (total_assets + 1 - fee_underlying)
```

## Yield Flow

```
1. User deposits USDC              -> vault idle balance increases, shares minted
2. Admin calls allocate(amount)    -> CPI klend::deposit, idle decreases, total_invested increases
3. Borrowers pay interest in klend -> klend share value grows
4. Admin calls harvest()           -> reads klend state, computes yield, mints fee shares
5. Admin calls deallocate(shares)  -> CPI klend::withdraw, idle increases, total_invested decreases
6. User calls withdraw(shares)     -> burns shares, receives USDC from idle balance
```

## Key Design Decisions

| Decision | Choice | Rationale |
|---|---|---|
| Share tokens | SPL mint PDA (6 decimals) | Fungible, transferable, composable (like ERC-4626) |
| Invested tracking | Cached `total_invested` field | Avoids reading klend state on every deposit/withdraw |
| Yield accounting | Harvest reads klend state, updates cache | Yearn V3 pattern, admin-triggered |
| Fees | Dilutive share minting | No token transfers, standard vault pattern |
| Withdrawal source | Idle only (no auto-deallocate) | Simpler, avoids CPI complexity on user path |
| Emergency | Halt flag blocks deposits, not withdrawals | Users must always be able to exit |
| Virtual shares | +1 offset | ERC-4626 inflation attack defense |
| Rounding | Always favor protocol (round down) | Standard vault safety |
| Deallocate accounting | `checked_sub` (not `saturating_sub`) | Errors on underflow rather than silent data corruption |

## CPI Dependencies

kvault calls into klend via Anchor CPI (`features = ["cpi"]`). klend must be built before kvault.

**allocate** maps to `klend::deposit`:
```
vault_authority     -> user (Signer via invoke_signed)
lending_market      -> lending_market
klend_reserve       -> reserve
klend_obligation    -> obligation
vault_token_account -> user_token_account
klend_token_vault   -> token_vault
```

**deallocate** maps to `klend::withdraw`:
```
vault_authority         -> user (Signer)
lending_market          -> lending_market
klend_reserve           -> reserve
klend_reserve_authority -> reserve_authority
klend_obligation        -> obligation
vault_authority         -> owner
vault_token_account     -> user_token_account
klend_token_vault       -> token_vault
klend_oracle            -> oracle
```

## Build & Test

```bash
# klend must be built first (CPI dependency)
cd ../klend && anchor build
cd ../kvault && anchor build
cd tests-litesvm && cargo test
```

12 tests covering vault init, deposits, proportional share allocation, allocate/deallocate to klend, yield harvest, exchange rate increase after yield, withdrawals, emergency halt, deposit cap, and a full deposit-allocate-borrow-harvest-deallocate-withdraw lifecycle.

## Project Layout

```
programs/kvault/src/
  lib.rs              7 instruction dispatchers
  constants.rs        PDA seeds, BPS scale, virtual shares, authority funding
  errors.rs           KvaultError enum
  math.rs             amount_to_shares, shares_to_amount, fee_shares, klend_shares_to_underlying
  state.rs            Vault struct (290 bytes)
  instructions/
    init_vault.rs     Create vault, share mint, vault ATA, fund authority
    deposit.rs        Deposit underlying, mint shares
    withdraw.rs       Burn shares, withdraw underlying (works when halted)
    allocate.rs       CPI klend::deposit (idle -> klend)
    deallocate.rs     CPI klend::withdraw (klend -> idle)
    harvest.rs        Compute yield from klend state, mint fee shares
    set_halt.rs       Emergency halt toggle
tests-litesvm/src/
  lib.rs              12 LiteSVM integration tests (loads both klend + kvault .so)
```
