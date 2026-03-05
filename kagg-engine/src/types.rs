use kagg::types::RoutePlanStep;
use solana_sdk::pubkey::Pubkey;

/// One leg within a route hop (single pool swap).
#[derive(Debug, Clone)]
pub struct RouteLeg {
    pub pool_index: usize,
    pub a_to_b: bool,
    pub amount_in: u64, // 0 = use previous output
    pub expected_out: u64,
}

/// One hop in the route. Multiple legs = split across pools.
#[derive(Debug, Clone)]
pub struct RouteHop {
    pub legs: Vec<RouteLeg>, // 1 = simple, >1 = split
}

/// Complete route from input to output.
#[derive(Debug, Clone)]
pub struct Route {
    pub hops: Vec<RouteHop>,
    pub input_mint: Pubkey,
    pub output_mint: Pubkey,
    pub amount_in: u64,
    pub expected_out: u64,
}

/// Output of the route plan builder — ready for on-chain execution.
#[derive(Clone)]
pub struct RoutePlanOutput {
    pub route_plan: Vec<RoutePlanStep>,
    /// (key, is_signer, is_writable) for each remaining account
    pub remaining_accounts: Vec<(Pubkey, bool, bool)>,
    pub token_ledger_len: u8,
    pub intermediate_mints: Vec<Pubkey>,
}
