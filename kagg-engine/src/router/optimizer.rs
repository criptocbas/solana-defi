use solana_sdk::pubkey::Pubkey;
use crate::graph::TokenGraph;
use crate::pool::QuotablePool;
use crate::router::pathfinder::{enumerate_paths, quote_path, CandidatePath};
use crate::types::{Route, RouteHop, RouteLeg};

/// Find the best route from input_mint to output_mint across all available pools.
///
/// Algorithm:
/// 1. Enumerate candidate paths via BFS
/// 2. Quote each path for the full amount, rank by output
/// 3. For top 2 candidates: try 2-way split optimization via golden-section search
/// 4. Return the best Route (single path or split)
pub fn find_best_route(
    pools: &[Box<dyn QuotablePool>],
    graph: &TokenGraph,
    input_mint: &Pubkey,
    output_mint: &Pubkey,
    amount_in: u64,
    max_hops: usize,
    max_splits: usize,
) -> Option<Route> {
    let candidates = enumerate_paths(graph, pools, input_mint, output_mint, max_hops, 20);
    if candidates.is_empty() {
        return None;
    }

    // Quote all candidates and sort by output (descending)
    let mut quoted: Vec<(usize, u64)> = candidates
        .iter()
        .enumerate()
        .filter_map(|(i, path)| {
            quote_path(path, pools, amount_in).map(|out| (i, out))
        })
        .collect();

    if quoted.is_empty() {
        return None;
    }

    quoted.sort_by(|a, b| b.1.cmp(&a.1));

    let best_single_idx = quoted[0].0;
    let best_single_out = quoted[0].1;

    // Try split optimization if we have at least 2 candidates and max_splits >= 2
    let mut best_route = build_single_route(
        &candidates[best_single_idx],
        pools,
        *input_mint,
        *output_mint,
        amount_in,
        best_single_out,
    );

    if max_splits >= 2 && quoted.len() >= 2 {
        let path_a = &candidates[quoted[0].0];
        let path_b = &candidates[quoted[1].0];

        if let Some((ratio, split_out)) =
            golden_section_split(path_a, path_b, pools, amount_in)
        {
            if split_out > best_single_out {
                let amount_a = ((amount_in as u128) * (ratio as u128) / 10000) as u64;
                let amount_b = amount_in - amount_a;

                let out_a = quote_path(path_a, pools, amount_a).unwrap_or(0);
                let out_b = quote_path(path_b, pools, amount_b).unwrap_or(0);

                best_route = build_split_route(
                    path_a,
                    path_b,
                    pools,
                    *input_mint,
                    *output_mint,
                    amount_a,
                    amount_b,
                    out_a,
                    out_b,
                );
            }
        }
    }

    Some(best_route)
}

/// Golden-section search on split ratio [0, 10000] (basis points).
/// Returns (optimal_ratio_bps, total_output).
fn golden_section_split(
    path_a: &CandidatePath,
    path_b: &CandidatePath,
    pools: &[Box<dyn QuotablePool>],
    total_amount: u64,
) -> Option<(u64, u64)> {
    let gr: f64 = (5.0_f64.sqrt() - 1.0) / 2.0; // golden ratio conjugate ≈ 0.618

    let eval = |ratio_bps: u64| -> u64 {
        let amount_a = ((total_amount as u128) * (ratio_bps as u128) / 10000) as u64;
        let amount_b = total_amount.saturating_sub(amount_a);
        let out_a = if amount_a > 0 {
            quote_path(path_a, pools, amount_a).unwrap_or(0)
        } else {
            0
        };
        let out_b = if amount_b > 0 {
            quote_path(path_b, pools, amount_b).unwrap_or(0)
        } else {
            0
        };
        out_a + out_b
    };

    let mut a: f64 = 0.0;
    let mut b: f64 = 10000.0;

    let mut c = b - gr * (b - a);
    let mut d = a + gr * (b - a);

    for _ in 0..25 {
        let fc = eval(c as u64);
        let fd = eval(d as u64);

        if fc > fd {
            b = d;
        } else {
            a = c;
        }

        c = b - gr * (b - a);
        d = a + gr * (b - a);

        if (b - a) < 1.0 {
            break;
        }
    }

    let optimal_bps = ((a + b) / 2.0) as u64;
    let total_out = eval(optimal_bps);
    Some((optimal_bps, total_out))
}

fn build_single_route(
    path: &CandidatePath,
    pools: &[Box<dyn QuotablePool>],
    input_mint: Pubkey,
    output_mint: Pubkey,
    amount_in: u64,
    expected_out: u64,
) -> Route {
    let mut hops = Vec::new();
    let mut current_amount = amount_in;

    for (i, &(pool_index, a_to_b)) in path.hops.iter().enumerate() {
        let out = pools[pool_index]
            .quote(current_amount, a_to_b)
            .unwrap_or(0);
        hops.push(RouteHop {
            legs: vec![RouteLeg {
                pool_index,
                a_to_b,
                amount_in: if i == 0 { amount_in } else { 0 },
                expected_out: out,
            }],
        });
        current_amount = out;
    }

    Route {
        hops,
        input_mint,
        output_mint,
        amount_in,
        expected_out,
    }
}

fn build_split_route(
    path_a: &CandidatePath,
    path_b: &CandidatePath,
    pools: &[Box<dyn QuotablePool>],
    input_mint: Pubkey,
    output_mint: Pubkey,
    amount_a: u64,
    amount_b: u64,
    out_a: u64,
    out_b: u64,
) -> Route {
    // For a split, we create one hop with multiple legs (one per path).
    // If paths are multi-hop, we need to serialize them sequentially.
    // Simple case: both paths are single-hop → one RouteHop with 2 legs.
    // Complex case: paths differ in length → serialize as separate hops with explicit amounts.

    let mut hops = Vec::new();

    // Path A hops
    let mut current_a = amount_a;
    for (i, &(pool_index, a_to_b)) in path_a.hops.iter().enumerate() {
        let out = pools[pool_index]
            .quote(current_a, a_to_b)
            .unwrap_or(0);
        hops.push(RouteHop {
            legs: vec![RouteLeg {
                pool_index,
                a_to_b,
                amount_in: if i == 0 { amount_a } else { 0 },
                expected_out: out,
            }],
        });
        current_a = out;
    }

    // Path B hops
    let mut current_b = amount_b;
    for (i, &(pool_index, a_to_b)) in path_b.hops.iter().enumerate() {
        let out = pools[pool_index]
            .quote(current_b, a_to_b)
            .unwrap_or(0);
        hops.push(RouteHop {
            legs: vec![RouteLeg {
                pool_index,
                a_to_b,
                amount_in: if i == 0 { amount_b } else { 0 },
                expected_out: out,
            }],
        });
        current_b = out;
    }

    Route {
        hops,
        input_mint,
        output_mint,
        amount_in: amount_a + amount_b,
        expected_out: out_a + out_b,
    }
}
