use kagg::types::DexId;
use kagg_engine::*;
use kagg_engine::pool::{ClmmPool, CpammPool, QuotablePool, TickArrayData, TickData};
use kclmm::constants::*;
use kclmm::math;
use solana_sdk::pubkey::Pubkey;

// ============================================================================
// Helpers
// ============================================================================

fn random_pubkey(seed: u8) -> Pubkey {
    let mut bytes = [0u8; 32];
    bytes[0] = seed;
    Pubkey::new_from_array(bytes)
}

fn make_cpamm(
    seed: u8,
    mint_a: Pubkey,
    mint_b: Pubkey,
    reserve_a: u64,
    reserve_b: u64,
) -> CpammPool {
    CpammPool {
        address: random_pubkey(seed),
        mint_a,
        mint_b,
        vault_a: random_pubkey(seed + 100),
        vault_b: random_pubkey(seed + 101),
        authority: random_pubkey(seed + 102),
        reserve_a,
        reserve_b,
        fee_numerator: 30,
        fee_denominator: 10_000,
        program_id: random_pubkey(seed + 103),
    }
}

/// Create a CLMM pool with a single tick range of uniform liquidity.
/// `lower_tick` and `upper_tick` define the range, `liquidity` is the active L.
/// `current_tick` sets where the price is.
fn make_clmm_single_range(
    seed: u8,
    mint_a: Pubkey,
    mint_b: Pubkey,
    lower_tick: i32,
    upper_tick: i32,
    liquidity: u128,
    current_tick: i32,
    fee_rate: u32,
    tick_spacing: u16,
) -> ClmmPool {
    let sqrt_price = math::tick_to_sqrt_price(current_tick).unwrap();

    // Build tick array(s) covering lower_tick and upper_tick
    let lower_array_start = math::tick_array_start_for_tick(lower_tick, tick_spacing);
    let upper_array_start = math::tick_array_start_for_tick(upper_tick, tick_spacing);

    let mut arrays = Vec::new();

    // We may need multiple tick arrays
    let starts: Vec<i32> = if lower_array_start == upper_array_start {
        vec![lower_array_start]
    } else {
        vec![lower_array_start, upper_array_start]
    };

    for &start in &starts {
        let mut bitmap: u64 = 0;
        let mut ticks = vec![TickData::default(); TICKS_PER_ARRAY];

        // Set lower tick if in this array
        if let Some(idx) = math::tick_index_in_array(lower_tick, start, tick_spacing) {
            math::set_bit(&mut bitmap, idx);
            ticks[idx] = TickData {
                liquidity_net: liquidity as i128,
                liquidity_gross: liquidity,
            };
        }

        // Set upper tick if in this array
        if let Some(idx) = math::tick_index_in_array(upper_tick, start, tick_spacing) {
            math::set_bit(&mut bitmap, idx);
            ticks[idx] = TickData {
                liquidity_net: -(liquidity as i128),
                liquidity_gross: liquidity,
            };
        }

        arrays.push(TickArrayData {
            address: random_pubkey(seed + 50 + arrays.len() as u8),
            start_tick_index: start,
            initialized_bitmap: bitmap,
            ticks,
        });
    }

    // Sort arrays by start_tick_index descending for a_to_b, ascending for b_to_a.
    // We'll keep ascending and let the search handle direction.
    arrays.sort_by_key(|a| a.start_tick_index);

    ClmmPool {
        address: random_pubkey(seed),
        mint_a,
        mint_b,
        vault_a: random_pubkey(seed + 110),
        vault_b: random_pubkey(seed + 111),
        authority: random_pubkey(seed + 112),
        sqrt_price,
        tick_current: current_tick,
        liquidity,
        fee_rate,
        tick_spacing,
        tick_arrays: arrays,
        program_id: random_pubkey(seed + 113),
    }
}

// ============================================================================
// Layer 1: Pool quoting tests (1-8)
// ============================================================================

#[test]
fn test_01_cpamm_quote_a_to_b() {
    let mint_a = random_pubkey(1);
    let mint_b = random_pubkey(2);
    let pool = make_cpamm(10, mint_a, mint_b, 1_000_000, 1_000_000);

    let out = pool.quote(10_000, true).unwrap();

    // Manual: amount_with_fee = 10000 * 9970 = 99_700_000
    // numerator = 1_000_000 * 99_700_000 = 99_700_000_000_000
    // denominator = 1_000_000 * 10_000 + 99_700_000 = 10_099_700_000
    // out = 99_700_000_000_000 / 10_099_700_000 = 9870
    assert_eq!(out, 9871);
}

#[test]
fn test_02_cpamm_quote_b_to_a() {
    let mint_a = random_pubkey(1);
    let mint_b = random_pubkey(2);
    let pool = make_cpamm(10, mint_a, mint_b, 1_000_000, 2_000_000);

    let out = pool.quote(50_000, false).unwrap();

    // B→A: reserve_in=2M, reserve_out=1M
    // amount_with_fee = 50_000 * 9970 = 498_500_000
    // numerator = 1_000_000 * 498_500_000 = 498_500_000_000_000
    // denominator = 2_000_000 * 10_000 + 498_500_000 = 20_498_500_000
    // out = 498_500_000_000_000 / 20_498_500_000 = 24_319
    assert_eq!(out, 24_318);
}

#[test]
fn test_03_cpamm_quote_large_amount() {
    let mint_a = random_pubkey(1);
    let mint_b = random_pubkey(2);
    let pool = make_cpamm(10, mint_a, mint_b, 1_000_000, 1_000_000);

    // Swap 500k into a 1M/1M pool — significant price impact
    let out = pool.quote(500_000, true).unwrap();
    // Should get less than 500k due to impact + fee
    assert!(out < 500_000);
    assert!(out > 300_000); // but more than 300k
}

#[test]
fn test_04_cpamm_quote_zero_reserves() {
    let mint_a = random_pubkey(1);
    let mint_b = random_pubkey(2);
    let pool = make_cpamm(10, mint_a, mint_b, 0, 1_000_000);

    assert!(pool.quote(10_000, true).is_none());
}

#[test]
fn test_05_clmm_quote_within_single_tick_range() {
    let mint_a = random_pubkey(1);
    let mint_b = random_pubkey(2);

    // tick_spacing=60, position from tick -120 to 120
    // current_tick = 0 (price = 1.0)
    // High liquidity so a small swap stays within range
    let pool = make_clmm_single_range(
        20, mint_a, mint_b,
        -120, 120,
        1_000_000_000_000, // 1T liquidity
        0,                 // current tick
        3000,              // 0.3% fee
        60,                // tick spacing
    );

    let out = pool.quote(1_000, true).unwrap();
    // With very high liquidity, output should be close to input minus fee
    // Fee = 0.3%, so expect ~997
    assert!(out >= 990 && out <= 999, "out={}", out);
}

#[test]
fn test_06_clmm_quote_crossing_one_tick() {
    let mint_a = random_pubkey(1);
    let mint_b = random_pubkey(2);

    // Create a pool with moderate liquidity where a larger swap crosses a tick
    // Position from -600 to 600, tick_spacing=60
    let pool = make_clmm_single_range(
        21, mint_a, mint_b,
        -600, 600,
        1_000_000_000, // 1B liquidity
        0,
        3000,
        60,
    );

    // Small swap within range
    let out_small = pool.quote(10_000, true).unwrap();
    // Larger swap that may push past a tick
    let out_large = pool.quote(1_000_000, true).unwrap();

    assert!(out_small > 0);
    assert!(out_large > 0);
    // Larger swap should have worse effective price (more price impact)
    let rate_small = out_small as f64 / 10_000.0;
    let rate_large = out_large as f64 / 1_000_000.0;
    assert!(
        rate_large <= rate_small,
        "larger swap should have worse price: large_rate={}, small_rate={}",
        rate_large,
        rate_small,
    );
}

#[test]
fn test_07_clmm_quote_crossing_multiple_ticks() {
    let mint_a = random_pubkey(1);
    let mint_b = random_pubkey(2);

    // Create two contiguous positions with different liquidity
    let tick_spacing: u16 = 60;
    let mut arrays = Vec::new();

    // Build arrays with two positions:
    // Position 1: [-600, 0] with liquidity 500M
    // Position 2: [0, 600] with liquidity 2B
    // We need to combine liquidity_net at tick 0

    let all_starts: Vec<i32> = {
        let mut s = vec![
            math::tick_array_start_for_tick(-600, tick_spacing),
            math::tick_array_start_for_tick(0, tick_spacing),
            math::tick_array_start_for_tick(600, tick_spacing),
        ];
        s.sort();
        s.dedup();
        s
    };

    let liq_1: u128 = 500_000_000;
    let liq_2: u128 = 2_000_000_000;

    for &start in &all_starts {
        let mut bitmap: u64 = 0;
        let mut ticks = vec![TickData::default(); TICKS_PER_ARRAY];

        // Tick -600: lower of position 1 → +liq_1
        if let Some(idx) = math::tick_index_in_array(-600, start, tick_spacing) {
            math::set_bit(&mut bitmap, idx);
            ticks[idx].liquidity_net += liq_1 as i128;
            ticks[idx].liquidity_gross += liq_1;
        }
        // Tick 0: upper of position 1 → -liq_1, lower of position 2 → +liq_2
        // Net = -liq_1 + liq_2
        if let Some(idx) = math::tick_index_in_array(0, start, tick_spacing) {
            math::set_bit(&mut bitmap, idx);
            ticks[idx].liquidity_net += -(liq_1 as i128) + liq_2 as i128;
            ticks[idx].liquidity_gross += liq_1 + liq_2;
        }
        // Tick 600: upper of position 2 → -liq_2
        if let Some(idx) = math::tick_index_in_array(600, start, tick_spacing) {
            math::set_bit(&mut bitmap, idx);
            ticks[idx].liquidity_net += -(liq_2 as i128);
            ticks[idx].liquidity_gross += liq_2;
        }

        if bitmap != 0 {
            arrays.push(TickArrayData {
                address: random_pubkey(30 + arrays.len() as u8),
                start_tick_index: start,
                initialized_bitmap: bitmap,
                ticks,
            });
        }
    }

    arrays.sort_by_key(|a| a.start_tick_index);

    // Current tick is 300 (in position 2), so active liquidity = liq_2
    let current_tick = 300;
    let pool = ClmmPool {
        address: random_pubkey(25),
        mint_a,
        mint_b,
        vault_a: random_pubkey(126),
        vault_b: random_pubkey(127),
        authority: random_pubkey(128),
        sqrt_price: math::tick_to_sqrt_price(current_tick).unwrap(),
        tick_current: current_tick,
        liquidity: liq_2,
        fee_rate: 3000,
        tick_spacing,
        tick_arrays: arrays,
        program_id: random_pubkey(129),
    };

    // b_to_a swap (price increases, moves right through ticks)
    let out = pool.quote(10_000, false);
    assert!(out.is_some(), "should produce output for b_to_a swap");
    assert!(out.unwrap() > 0);
}

#[test]
fn test_08_clmm_quote_zero_liquidity() {
    let mint_a = random_pubkey(1);
    let mint_b = random_pubkey(2);

    // Pool with 0 liquidity
    let pool = ClmmPool {
        address: random_pubkey(30),
        mint_a,
        mint_b,
        vault_a: random_pubkey(130),
        vault_b: random_pubkey(131),
        authority: random_pubkey(132),
        sqrt_price: Q64,
        tick_current: 0,
        liquidity: 0,
        fee_rate: 3000,
        tick_spacing: 60,
        tick_arrays: vec![],
        program_id: random_pubkey(133),
    };

    assert!(pool.quote(1_000, true).is_none());
}

// ============================================================================
// Layer 2: Token graph + pathfinding tests (9-14)
// ============================================================================

#[test]
fn test_09_build_graph_adjacency() {
    let mint_a = random_pubkey(1);
    let mint_b = random_pubkey(2);
    let mint_c = random_pubkey(3);

    let pools: Vec<Box<dyn QuotablePool>> = vec![
        Box::new(make_cpamm(10, mint_a, mint_b, 1_000_000, 1_000_000)),
        Box::new(make_cpamm(11, mint_b, mint_c, 1_000_000, 1_000_000)),
        Box::new(make_cpamm(12, mint_a, mint_c, 1_000_000, 1_000_000)),
    ];

    let graph = TokenGraph::build(&pools);

    // mint_a should have 4 edges (A→B via pool0, A→C via pool2, + reverse edges appear on other mints)
    assert_eq!(graph.neighbors(&mint_a).len(), 2); // A→B, A→C
    assert_eq!(graph.neighbors(&mint_b).len(), 2); // B→A, B→C
    assert_eq!(graph.neighbors(&mint_c).len(), 2); // C→A, C→B
}

#[test]
fn test_10_path_enumeration_direct() {
    let mint_a = random_pubkey(1);
    let mint_b = random_pubkey(2);

    let pools: Vec<Box<dyn QuotablePool>> = vec![
        Box::new(make_cpamm(10, mint_a, mint_b, 1_000_000, 1_000_000)),
    ];

    let graph = TokenGraph::build(&pools);
    let paths = enumerate_paths(&graph, &pools, &mint_a, &mint_b, 4, 20);

    assert_eq!(paths.len(), 1);
    assert_eq!(paths[0].hops.len(), 1);
    assert_eq!(paths[0].hops[0], (0, true)); // pool 0, a_to_b
}

#[test]
fn test_11_path_enumeration_2hop() {
    let mint_a = random_pubkey(1);
    let mint_b = random_pubkey(2);
    let mint_c = random_pubkey(3);

    let pools: Vec<Box<dyn QuotablePool>> = vec![
        Box::new(make_cpamm(10, mint_a, mint_b, 1_000_000, 1_000_000)),
        Box::new(make_cpamm(11, mint_b, mint_c, 1_000_000, 1_000_000)),
    ];

    let graph = TokenGraph::build(&pools);
    let paths = enumerate_paths(&graph, &pools, &mint_a, &mint_c, 4, 20);

    assert_eq!(paths.len(), 1);
    assert_eq!(paths[0].hops.len(), 2);
}

#[test]
fn test_12_path_enumeration_triangle() {
    let mint_a = random_pubkey(1);
    let mint_b = random_pubkey(2);
    let mint_c = random_pubkey(3);

    let pools: Vec<Box<dyn QuotablePool>> = vec![
        Box::new(make_cpamm(10, mint_a, mint_b, 1_000_000, 1_000_000)),
        Box::new(make_cpamm(11, mint_b, mint_c, 1_000_000, 1_000_000)),
        Box::new(make_cpamm(12, mint_a, mint_c, 1_000_000, 1_000_000)),
    ];

    let graph = TokenGraph::build(&pools);
    let paths = enumerate_paths(&graph, &pools, &mint_a, &mint_c, 4, 20);

    // Should find: A→C direct (1 hop) AND A→B→C (2 hops)
    assert_eq!(paths.len(), 2);
    let hop_counts: Vec<usize> = paths.iter().map(|p| p.hops.len()).collect();
    assert!(hop_counts.contains(&1));
    assert!(hop_counts.contains(&2));
}

#[test]
fn test_13_path_enumeration_max_hops() {
    let mint_a = random_pubkey(1);
    let mint_b = random_pubkey(2);
    let mint_c = random_pubkey(3);
    let mint_d = random_pubkey(4);

    let pools: Vec<Box<dyn QuotablePool>> = vec![
        Box::new(make_cpamm(10, mint_a, mint_b, 1_000_000, 1_000_000)),
        Box::new(make_cpamm(11, mint_b, mint_c, 1_000_000, 1_000_000)),
        Box::new(make_cpamm(12, mint_c, mint_d, 1_000_000, 1_000_000)),
    ];

    let graph = TokenGraph::build(&pools);

    // With max_hops=2, should NOT find A→B→C→D (3 hops)
    let paths = enumerate_paths(&graph, &pools, &mint_a, &mint_d, 2, 20);
    assert_eq!(paths.len(), 0);

    // With max_hops=3, should find it
    let paths = enumerate_paths(&graph, &pools, &mint_a, &mint_d, 3, 20);
    assert_eq!(paths.len(), 1);
    assert_eq!(paths[0].hops.len(), 3);
}

#[test]
fn test_14_multihop_quote() {
    let mint_a = random_pubkey(1);
    let mint_b = random_pubkey(2);
    let mint_c = random_pubkey(3);

    let pool_ab = make_cpamm(10, mint_a, mint_b, 1_000_000, 2_000_000);
    let pool_bc = make_cpamm(11, mint_b, mint_c, 2_000_000, 1_000_000);

    // Manual sequential quote
    let mid = pool_ab.quote(10_000, true).unwrap();
    let expected = pool_bc.quote(mid, true).unwrap();

    let pools: Vec<Box<dyn QuotablePool>> = vec![
        Box::new(pool_ab),
        Box::new(pool_bc),
    ];

    let graph = TokenGraph::build(&pools);
    let paths = enumerate_paths(&graph, &pools, &mint_a, &mint_c, 4, 20);
    assert_eq!(paths.len(), 1);

    let actual = quote_path(&paths[0], &pools, 10_000).unwrap();
    assert_eq!(actual, expected);
}

// ============================================================================
// Layer 3: Split optimization + RoutePlan builder tests (15-20)
// ============================================================================

#[test]
fn test_15_split_outperforms_single() {
    let mint_a = random_pubkey(1);
    let mint_b = random_pubkey(2);

    // Two pools with different depths — splitting should help
    let pools: Vec<Box<dyn QuotablePool>> = vec![
        Box::new(make_cpamm(10, mint_a, mint_b, 1_000_000, 1_000_000)),
        Box::new(make_cpamm(11, mint_a, mint_b, 2_000_000, 2_000_000)),
    ];

    let graph = TokenGraph::build(&pools);
    let amount = 500_000;

    // Best single pool
    let out_0 = pools[0].quote(amount, true).unwrap();
    let out_1 = pools[1].quote(amount, true).unwrap();
    let best_single = out_0.max(out_1);

    // find_best_route with splits
    let route = find_best_route(&pools, &graph, &mint_a, &mint_b, amount, 4, 2).unwrap();

    // Split should give better output than single
    assert!(
        route.expected_out >= best_single,
        "split={} should >= single={}",
        route.expected_out,
        best_single
    );
}

#[test]
fn test_16_split_one_pool_dominates() {
    let mint_a = random_pubkey(1);
    let mint_b = random_pubkey(2);

    // One pool is 100x deeper — should get ~100% of flow
    let pools: Vec<Box<dyn QuotablePool>> = vec![
        Box::new(make_cpamm(10, mint_a, mint_b, 100_000, 100_000)),          // shallow
        Box::new(make_cpamm(11, mint_a, mint_b, 10_000_000, 10_000_000)),    // deep
    ];

    let graph = TokenGraph::build(&pools);
    let amount = 1_000;

    let route = find_best_route(&pools, &graph, &mint_a, &mint_b, amount, 4, 2).unwrap();

    // The deep pool alone should give most of the output
    let deep_only = pools[1].quote(amount, true).unwrap();
    // Route should be at least as good as the deep pool alone
    assert!(route.expected_out >= deep_only);
}

#[test]
fn test_17_find_best_route_single_path() {
    let mint_a = random_pubkey(1);
    let mint_b = random_pubkey(2);
    let mint_c = random_pubkey(3);

    let pools: Vec<Box<dyn QuotablePool>> = vec![
        Box::new(make_cpamm(10, mint_a, mint_b, 1_000_000, 1_000_000)),
        Box::new(make_cpamm(11, mint_b, mint_c, 1_000_000, 1_000_000)),
    ];

    let graph = TokenGraph::build(&pools);
    let route = find_best_route(&pools, &graph, &mint_a, &mint_c, 10_000, 4, 2).unwrap();

    assert_eq!(route.input_mint, mint_a);
    assert_eq!(route.output_mint, mint_c);
    assert_eq!(route.amount_in, 10_000);
    assert!(route.expected_out > 0);
}

#[test]
fn test_18_route_plan_single_hop() {
    let mint_a = random_pubkey(1);
    let mint_b = random_pubkey(2);
    let user_source = random_pubkey(200);
    let user_dest = random_pubkey(201);

    let pools: Vec<Box<dyn QuotablePool>> = vec![
        Box::new(make_cpamm(10, mint_a, mint_b, 1_000_000, 1_000_000)),
    ];

    let graph = TokenGraph::build(&pools);
    let route = find_best_route(&pools, &graph, &mint_a, &mint_b, 10_000, 4, 2).unwrap();
    let plan = build_route_plan(&route, &pools, &user_source, &user_dest);

    assert_eq!(plan.route_plan.len(), 1);
    assert_eq!(plan.token_ledger_len, 0); // no intermediates
    assert_eq!(plan.intermediate_mints.len(), 0);

    let step = &plan.route_plan[0];
    assert_eq!(step.input_token_index, 0);  // source
    assert_eq!(step.output_token_index, 1); // destination
    assert_eq!(step.amount_in, 10_000);
    assert!(matches!(step.dex_id, DexId::Kpool));
}

#[test]
fn test_19_route_plan_2hop() {
    let mint_a = random_pubkey(1);
    let mint_b = random_pubkey(2);
    let mint_c = random_pubkey(3);
    let user_source = random_pubkey(200);
    let user_dest = random_pubkey(201);

    let pools: Vec<Box<dyn QuotablePool>> = vec![
        Box::new(make_cpamm(10, mint_a, mint_b, 1_000_000, 1_000_000)),
        Box::new(make_cpamm(11, mint_b, mint_c, 1_000_000, 1_000_000)),
    ];

    let graph = TokenGraph::build(&pools);
    let route = find_best_route(&pools, &graph, &mint_a, &mint_c, 10_000, 4, 2).unwrap();
    let plan = build_route_plan(&route, &pools, &user_source, &user_dest);

    assert_eq!(plan.route_plan.len(), 2);
    assert_eq!(plan.token_ledger_len, 1); // mint_b is intermediate
    assert_eq!(plan.intermediate_mints.len(), 1);
    assert_eq!(plan.intermediate_mints[0], mint_b);

    // Step 0: source(0) → intermediate_b(2)
    let s0 = &plan.route_plan[0];
    assert_eq!(s0.input_token_index, 0);
    assert_eq!(s0.output_token_index, 2);
    assert_eq!(s0.amount_in, 10_000);

    // Step 1: intermediate_b(2) → destination(1)
    let s1 = &plan.route_plan[1];
    assert_eq!(s1.input_token_index, 2);
    assert_eq!(s1.output_token_index, 1);
    assert_eq!(s1.amount_in, 0); // uses previous output
}

#[test]
fn test_20_route_plan_split() {
    let mint_a = random_pubkey(1);
    let mint_b = random_pubkey(2);
    let user_source = random_pubkey(200);
    let user_dest = random_pubkey(201);

    // Two pools — force a split by making them similar depth
    let pools: Vec<Box<dyn QuotablePool>> = vec![
        Box::new(make_cpamm(10, mint_a, mint_b, 1_000_000, 1_000_000)),
        Box::new(make_cpamm(11, mint_a, mint_b, 1_000_000, 1_000_000)),
    ];

    let graph = TokenGraph::build(&pools);
    let route = find_best_route(&pools, &graph, &mint_a, &mint_b, 500_000, 4, 2).unwrap();
    let plan = build_route_plan(&route, &pools, &user_source, &user_dest);

    // If split happened, we get 2 steps with explicit amounts
    if plan.route_plan.len() == 2 {
        let s0 = &plan.route_plan[0];
        let s1 = &plan.route_plan[1];
        // Both should have explicit amount_in
        assert!(s0.amount_in > 0, "first split leg should have explicit amount");
        assert!(s1.amount_in > 0, "second split leg should have explicit amount");
        // Total should equal input
        assert_eq!(s0.amount_in + s1.amount_in, 500_000);
        // Both should map source→dest
        assert_eq!(s0.input_token_index, 0);
        assert_eq!(s0.output_token_index, 1);
        assert_eq!(s1.input_token_index, 0);
        assert_eq!(s1.output_token_index, 1);
    } else {
        // Single path is also valid if optimizer decides split doesn't help
        assert_eq!(plan.route_plan.len(), 1);
    }
}
