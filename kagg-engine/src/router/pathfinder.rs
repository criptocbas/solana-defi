use std::collections::HashSet;
use std::collections::VecDeque;
use solana_sdk::pubkey::Pubkey;
use crate::graph::TokenGraph;
use crate::pool::QuotablePool;

/// A candidate path: sequence of (pool_index, a_to_b) hops.
#[derive(Debug, Clone)]
pub struct CandidatePath {
    pub hops: Vec<(usize, bool)>,
}

/// BFS enumeration of all paths from input_mint to output_mint.
/// Stops after finding max_candidates paths or exhausting all paths within max_hops.
pub fn enumerate_paths(
    graph: &TokenGraph,
    _pools: &[Box<dyn QuotablePool>],
    input_mint: &Pubkey,
    output_mint: &Pubkey,
    max_hops: usize,
    max_candidates: usize,
) -> Vec<CandidatePath> {
    let mut results = Vec::new();

    // BFS state: (current_mint, path_so_far, visited_mints)
    let mut queue: VecDeque<(Pubkey, Vec<(usize, bool)>, HashSet<Pubkey>)> = VecDeque::new();

    let mut initial_visited = HashSet::new();
    initial_visited.insert(*input_mint);
    queue.push_back((*input_mint, Vec::new(), initial_visited));

    while let Some((current_mint, path, visited)) = queue.pop_front() {
        if results.len() >= max_candidates {
            break;
        }
        if path.len() >= max_hops {
            continue;
        }

        for edge in graph.neighbors(&current_mint) {
            if results.len() >= max_candidates {
                break;
            }

            let next_mint = edge.other_mint;

            // Found a path to the destination
            if next_mint == *output_mint {
                let mut new_path = path.clone();
                new_path.push((edge.pool_index, edge.a_to_b));
                results.push(CandidatePath { hops: new_path });
                continue;
            }

            // Don't revisit mints (no cycles)
            if visited.contains(&next_mint) {
                continue;
            }

            // Enqueue for further exploration
            let mut new_path = path.clone();
            new_path.push((edge.pool_index, edge.a_to_b));
            let mut new_visited = visited.clone();
            new_visited.insert(next_mint);
            queue.push_back((next_mint, new_path, new_visited));
        }
    }

    results
}

/// Quote a candidate path: sequentially feed output of each hop as input to the next.
pub fn quote_path(
    path: &CandidatePath,
    pools: &[Box<dyn QuotablePool>],
    amount_in: u64,
) -> Option<u64> {
    let mut current_amount = amount_in;
    for &(pool_index, a_to_b) in &path.hops {
        current_amount = pools[pool_index].quote(current_amount, a_to_b)?;
    }
    if current_amount == 0 { None } else { Some(current_amount) }
}
