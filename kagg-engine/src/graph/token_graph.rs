use std::collections::HashMap;
use solana_sdk::pubkey::Pubkey;
use crate::pool::QuotablePool;

/// An edge in the token graph — one direction of a pool.
#[derive(Debug, Clone)]
pub struct PoolEdge {
    pub pool_index: usize,
    pub other_mint: Pubkey,
    pub a_to_b: bool,
}

/// Directed multigraph of tokens connected by pools.
pub struct TokenGraph {
    adjacency: HashMap<Pubkey, Vec<PoolEdge>>,
}

impl TokenGraph {
    /// Build the graph from a set of quotable pools.
    /// Each pool creates 2 edges: mint_a→mint_b (a_to_b=true) and mint_b→mint_a (a_to_b=false).
    pub fn build(pools: &[Box<dyn QuotablePool>]) -> Self {
        let mut adjacency: HashMap<Pubkey, Vec<PoolEdge>> = HashMap::new();

        for (i, pool) in pools.iter().enumerate() {
            let ma = pool.mint_a();
            let mb = pool.mint_b();

            // Edge: mint_a → mint_b
            adjacency.entry(ma).or_default().push(PoolEdge {
                pool_index: i,
                other_mint: mb,
                a_to_b: true,
            });

            // Edge: mint_b → mint_a
            adjacency.entry(mb).or_default().push(PoolEdge {
                pool_index: i,
                other_mint: ma,
                a_to_b: false,
            });
        }

        Self { adjacency }
    }

    /// Get outgoing edges from a token mint.
    pub fn neighbors(&self, mint: &Pubkey) -> &[PoolEdge] {
        self.adjacency.get(mint).map(|v| v.as_slice()).unwrap_or(&[])
    }
}
