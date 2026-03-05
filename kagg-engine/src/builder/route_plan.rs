use std::collections::HashMap;
use kagg::types::RoutePlanStep;
use solana_sdk::pubkey::Pubkey;
use crate::pool::QuotablePool;
use crate::types::{Route, RoutePlanOutput};

/// Convert an abstract Route into a concrete on-chain RoutePlan.
///
/// Token index mapping:
///   0 = user_source (input token account)
///   1 = user_destination (output token account)
///   2+ = intermediate token accounts from remaining_accounts[0..ledger_len]
///
/// remaining_accounts layout:
///   [intermediate_token_accounts..., step_0_accounts..., step_1_accounts..., ...]
pub fn build_route_plan(
    route: &Route,
    pools: &[Box<dyn QuotablePool>],
    _user_source: &Pubkey,
    _user_destination: &Pubkey,
) -> RoutePlanOutput {
    // Collect all intermediate mints (mints that are neither input nor output)
    let mut intermediate_mint_map: HashMap<Pubkey, u8> = HashMap::new();
    let mut intermediate_mints: Vec<Pubkey> = Vec::new();

    // Walk through hops to find intermediate mints
    for hop in &route.hops {
        for leg in &hop.legs {
            let pool = &pools[leg.pool_index];
            let (in_mint, out_mint) = if leg.a_to_b {
                (pool.mint_a(), pool.mint_b())
            } else {
                (pool.mint_b(), pool.mint_a())
            };

            // Register intermediate mints (not input or output)
            for mint in [in_mint, out_mint] {
                if mint != route.input_mint
                    && mint != route.output_mint
                    && !intermediate_mint_map.contains_key(&mint)
                {
                    let idx = 2 + intermediate_mints.len() as u8;
                    intermediate_mint_map.insert(mint, idx);
                    intermediate_mints.push(mint);
                }
            }
        }
    }

    let token_ledger_len = intermediate_mints.len() as u8;

    // Build RoutePlanSteps and collect remaining_accounts
    let mut route_plan: Vec<RoutePlanStep> = Vec::new();
    let mut step_accounts: Vec<(Pubkey, bool, bool)> = Vec::new();

    for hop in &route.hops {
        for leg in &hop.legs {
            let pool = &pools[leg.pool_index];
            let (in_mint, out_mint) = if leg.a_to_b {
                (pool.mint_a(), pool.mint_b())
            } else {
                (pool.mint_b(), pool.mint_a())
            };

            let input_token_index = mint_to_index(
                &in_mint,
                &route.input_mint,
                &route.output_mint,
                &intermediate_mint_map,
            );
            let output_token_index = mint_to_index(
                &out_mint,
                &route.input_mint,
                &route.output_mint,
                &intermediate_mint_map,
            );

            let accounts = pool.swap_accounts(leg.a_to_b);
            let num_accounts = accounts.len() as u8;

            route_plan.push(RoutePlanStep {
                dex_id: pool.dex_id(),
                num_accounts,
                amount_in: leg.amount_in,
                input_token_index,
                output_token_index,
                extra_data: pool.extra_data(leg.a_to_b, leg.amount_in),
            });

            step_accounts.extend(accounts);
        }
    }

    // remaining_accounts = [intermediate_token_accounts..., step_accounts...]
    // The intermediate token account pubkeys are placeholders here — the caller
    // needs to create/provide actual ATAs for these mints.
    let mut remaining_accounts: Vec<(Pubkey, bool, bool)> = Vec::new();

    // Placeholder entries for intermediate token accounts (will be ATAs)
    for mint in &intermediate_mints {
        remaining_accounts.push((*mint, false, true)); // writable ATA placeholder
    }

    // Append all step accounts
    remaining_accounts.extend(step_accounts);

    RoutePlanOutput {
        route_plan,
        remaining_accounts,
        token_ledger_len,
        intermediate_mints,
    }
}

fn mint_to_index(
    mint: &Pubkey,
    input_mint: &Pubkey,
    output_mint: &Pubkey,
    intermediate_map: &HashMap<Pubkey, u8>,
) -> u8 {
    if mint == input_mint {
        0
    } else if mint == output_mint {
        1
    } else {
        *intermediate_map.get(mint).expect("intermediate mint not found")
    }
}
