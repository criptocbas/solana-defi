#[cfg(test)]
mod tests {
    use anchor_lang::{system_program, InstructionData, ToAccountMetas};
    use anchor_spl::associated_token::get_associated_token_address;
    use litesvm::LiteSVM;
    use solana_sdk::{
        compute_budget::ComputeBudgetInstruction,
        instruction::{AccountMeta, Instruction},
        program_pack::Pack,
        pubkey::Pubkey,
        signature::{Keypair, Signer},
        transaction::Transaction,
    };
    use std::str::FromStr;

    use cpamm::constants as cpamm_c;
    use kclmm::constants as kclmm_c;
    use kclmm::math;

    const CPAMM_ID: &str = "8EpEqMJTjJwFPWbbaSsJi4bDM8z5eZp3aULqdaWppyr9";
    const KCLMM_ID: &str = "7g3bAmnUmaoXZcDxffmxsZk7hmhMNDcQw7pT2aNC5tYW";
    const KAGG_ID: &str = "3YJj1erVbKvjEJxJEMarh7dDYBQTrq7fNCcWdTPjuWLn";

    fn cpamm_id() -> Pubkey { Pubkey::from_str(CPAMM_ID).unwrap() }
    fn kclmm_id() -> Pubkey { Pubkey::from_str(KCLMM_ID).unwrap() }
    fn kagg_id() -> Pubkey { Pubkey::from_str(KAGG_ID).unwrap() }

    fn compute_budget_ix(units: u32) -> Instruction {
        ComputeBudgetInstruction::set_compute_unit_limit(units)
    }

    // ========== SPL helpers ==========

    fn create_mint(svm: &mut LiteSVM, payer: &Keypair, authority: &Pubkey, decimals: u8) -> Pubkey {
        let mint = Keypair::new();
        let rent = svm.minimum_balance_for_rent_exemption(spl_token::state::Mint::LEN);
        let create_ix = solana_sdk::system_instruction::create_account(
            &payer.pubkey(), &mint.pubkey(), rent,
            spl_token::state::Mint::LEN as u64, &spl_token::id(),
        );
        let init_ix = spl_token::instruction::initialize_mint(
            &spl_token::id(), &mint.pubkey(), authority, None, decimals,
        ).unwrap();
        let tx = Transaction::new_signed_with_payer(
            &[create_ix, init_ix], Some(&payer.pubkey()), &[payer, &mint], svm.latest_blockhash(),
        );
        svm.send_transaction(tx).unwrap();
        mint.pubkey()
    }

    fn create_ata(svm: &mut LiteSVM, payer: &Keypair, owner: &Pubkey, mint: &Pubkey) -> Pubkey {
        let ata = get_associated_token_address(owner, mint);
        let ix = spl_associated_token_account::instruction::create_associated_token_account(
            &payer.pubkey(), owner, mint, &spl_token::id(),
        );
        let tx = Transaction::new_signed_with_payer(
            &[ix], Some(&payer.pubkey()), &[payer], svm.latest_blockhash(),
        );
        svm.send_transaction(tx).unwrap();
        ata
    }

    fn create_ata_if_needed(svm: &mut LiteSVM, payer: &Keypair, owner: &Pubkey, mint: &Pubkey) -> Pubkey {
        let ata = get_associated_token_address(owner, mint);
        if svm.get_account(&ata).is_some() {
            return ata;
        }
        create_ata(svm, payer, owner, mint)
    }

    fn mint_tokens(svm: &mut LiteSVM, payer: &Keypair, mint: &Pubkey, dest: &Pubkey, authority: &Keypair, amount: u64) {
        let ix = spl_token::instruction::mint_to(
            &spl_token::id(), mint, dest, &authority.pubkey(), &[], amount,
        ).unwrap();
        let tx = Transaction::new_signed_with_payer(
            &[ix], Some(&payer.pubkey()), &[payer, authority], svm.latest_blockhash(),
        );
        svm.send_transaction(tx).unwrap();
    }

    fn token_balance(svm: &LiteSVM, account: &Pubkey) -> u64 {
        let data = svm.get_account(account).unwrap();
        spl_token::state::Account::unpack(&data.data).unwrap().amount
    }

    // ========== PDA helpers ==========

    fn cpamm_pool_pda(mint_a: &Pubkey, mint_b: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[cpamm_c::POOL_SEED, mint_a.as_ref(), mint_b.as_ref()],
            &cpamm_id(),
        )
    }

    fn cpamm_pool_authority_pda(pool: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[cpamm_c::POOL_AUTHORITY_SEED, pool.as_ref()],
            &cpamm_id(),
        )
    }

    fn cpamm_lp_mint_pda(pool: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[cpamm_c::LP_MINT_SEED, pool.as_ref()],
            &cpamm_id(),
        )
    }

    fn kclmm_pool_pda(mint_a: &Pubkey, mint_b: &Pubkey, fee_rate: u32) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[kclmm_c::POOL_SEED, mint_a.as_ref(), mint_b.as_ref(), &fee_rate.to_le_bytes()],
            &kclmm_id(),
        )
    }

    fn kclmm_pool_authority_pda(pool: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[kclmm_c::POOL_AUTHORITY_SEED, pool.as_ref()],
            &kclmm_id(),
        )
    }

    fn tick_array_pda(pool: &Pubkey, start_tick_index: i32) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[kclmm_c::TICK_ARRAY_SEED, pool.as_ref(), &start_tick_index.to_le_bytes()],
            &kclmm_id(),
        )
    }

    fn position_pda(pool: &Pubkey, owner: &Pubkey, tick_lower: i32, tick_upper: i32) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[kclmm_c::POSITION_SEED, pool.as_ref(), owner.as_ref(),
              &tick_lower.to_le_bytes(), &tick_upper.to_le_bytes()],
            &kclmm_id(),
        )
    }

    // ========== Pool info structs ==========

    struct KpoolInfo {
        pool: Pubkey,
        pool_authority: Pubkey,
        lp_mint: Pubkey,
        vault_a: Pubkey,
        vault_b: Pubkey,
        locked_lp_vault: Pubkey,
        mint_a: Pubkey,
        mint_b: Pubkey,
    }

    struct KclmmInfo {
        pool: Pubkey,
        pool_authority: Pubkey,
        vault_a: Pubkey,
        vault_b: Pubkey,
        mint_a: Pubkey,
        mint_b: Pubkey,
        tick_spacing: u16,
        tick_array_starts: Vec<i32>,
    }

    // ========== Pool setup ==========

    fn init_cpamm_pool(svm: &mut LiteSVM, payer: &Keypair, mint_a: &Pubkey, mint_b: &Pubkey) -> KpoolInfo {
        let (pool, _) = cpamm_pool_pda(mint_a, mint_b);
        let (pool_authority, _) = cpamm_pool_authority_pda(&pool);
        let (lp_mint, _) = cpamm_lp_mint_pda(&pool);
        let vault_a = get_associated_token_address(&pool_authority, mint_a);
        let vault_b = get_associated_token_address(&pool_authority, mint_b);
        let locked_lp_vault = get_associated_token_address(&pool_authority, &lp_mint);

        let ix = Instruction {
            program_id: cpamm_id(),
            accounts: cpamm::accounts::InitializePool {
                payer: payer.pubkey(),
                mint_a: *mint_a,
                mint_b: *mint_b,
                pool,
                pool_authority,
                vault_a,
                vault_b,
                lp_mint,
                locked_lp_vault,
                system_program: system_program::ID,
                token_program: spl_token::id(),
                associated_token_program: spl_associated_token_account::id(),
            }.to_account_metas(None),
            data: cpamm::instruction::InitializePool {}.data(),
        };
        let tx = Transaction::new_signed_with_payer(
            &[ix], Some(&payer.pubkey()), &[payer], svm.latest_blockhash(),
        );
        svm.send_transaction(tx).unwrap();

        KpoolInfo {
            pool, pool_authority, lp_mint, vault_a, vault_b, locked_lp_vault,
            mint_a: *mint_a, mint_b: *mint_b,
        }
    }

    fn add_cpamm_liquidity(svm: &mut LiteSVM, payer: &Keypair, info: &KpoolInfo, amount_a: u64, amount_b: u64) {
        let user_token_a = get_associated_token_address(&payer.pubkey(), &info.mint_a);
        let user_token_b = get_associated_token_address(&payer.pubkey(), &info.mint_b);
        let user_lp_token = create_ata_if_needed(svm, payer, &payer.pubkey(), &info.lp_mint);

        svm.expire_blockhash();
        let ix = Instruction {
            program_id: cpamm_id(),
            accounts: cpamm::accounts::AddLiquidity {
                user: payer.pubkey(),
                pool: info.pool,
                pool_authority: info.pool_authority,
                lp_mint: info.lp_mint,
                vault_a: info.vault_a,
                vault_b: info.vault_b,
                user_token_a,
                user_token_b,
                user_lp_token,
                locked_lp_vault: info.locked_lp_vault,
                system_program: system_program::ID,
                token_program: spl_token::id(),
                associated_token_program: spl_associated_token_account::id(),
            }.to_account_metas(None),
            data: cpamm::instruction::AddLiquidity {
                amount_a_desired: amount_a,
                amount_b_desired: amount_b,
                minimum_lp_tokens: 0,
            }.data(),
        };
        let tx = Transaction::new_signed_with_payer(
            &[ix], Some(&payer.pubkey()), &[payer], svm.latest_blockhash(),
        );
        svm.send_transaction(tx).unwrap();
    }

    fn init_kclmm_pool(
        svm: &mut LiteSVM, payer: &Keypair,
        mint_a: &Pubkey, mint_b: &Pubkey,
        fee_rate: u32, initial_sqrt_price: u128,
    ) -> KclmmInfo {
        let tick_spacing = kclmm_c::fee_rate_to_tick_spacing(fee_rate).unwrap();
        let (pool, _) = kclmm_pool_pda(mint_a, mint_b, fee_rate);
        let (pool_authority, _) = kclmm_pool_authority_pda(&pool);
        let vault_a = get_associated_token_address(&pool_authority, mint_a);
        let vault_b = get_associated_token_address(&pool_authority, mint_b);

        let ix = Instruction {
            program_id: kclmm_id(),
            accounts: kclmm::accounts::InitPool {
                payer: payer.pubkey(),
                mint_a: *mint_a,
                mint_b: *mint_b,
                pool,
                pool_authority,
                vault_a,
                vault_b,
                system_program: system_program::ID,
                token_program: spl_token::id(),
                associated_token_program: spl_associated_token_account::id(),
            }.to_account_metas(None),
            data: kclmm::instruction::InitPool { fee_rate, initial_sqrt_price }.data(),
        };
        let tx = Transaction::new_signed_with_payer(
            &[compute_budget_ix(1_400_000), ix],
            Some(&payer.pubkey()), &[payer], svm.latest_blockhash(),
        );
        svm.send_transaction(tx).unwrap();

        KclmmInfo {
            pool, pool_authority, vault_a, vault_b,
            mint_a: *mint_a, mint_b: *mint_b,
            tick_spacing, tick_array_starts: vec![],
        }
    }

    fn init_tick_arrays_and_add_liquidity(
        svm: &mut LiteSVM, payer: &Keypair, info: &mut KclmmInfo,
        tick_lower: i32, tick_upper: i32, liquidity: u128,
    ) {
        let ta_lower_start = math::tick_array_start_for_tick(tick_lower, info.tick_spacing);
        let ta_upper_start = math::tick_array_start_for_tick(tick_upper, info.tick_spacing);

        svm.expire_blockhash();
        let mut ixs = vec![compute_budget_ix(1_400_000)];

        let (ta_lower, _) = tick_array_pda(&info.pool, ta_lower_start);
        ixs.push(Instruction {
            program_id: kclmm_id(),
            accounts: kclmm::accounts::InitTickArray {
                payer: payer.pubkey(),
                pool: info.pool,
                tick_array: ta_lower,
                system_program: system_program::ID,
            }.to_account_metas(None),
            data: kclmm::instruction::InitTickArray { start_tick_index: ta_lower_start }.data(),
        });
        info.tick_array_starts.push(ta_lower_start);

        if ta_upper_start != ta_lower_start {
            let (ta_upper, _) = tick_array_pda(&info.pool, ta_upper_start);
            ixs.push(Instruction {
                program_id: kclmm_id(),
                accounts: kclmm::accounts::InitTickArray {
                    payer: payer.pubkey(),
                    pool: info.pool,
                    tick_array: ta_upper,
                    system_program: system_program::ID,
                }.to_account_metas(None),
                data: kclmm::instruction::InitTickArray { start_tick_index: ta_upper_start }.data(),
            });
            info.tick_array_starts.push(ta_upper_start);
        }

        let tx = Transaction::new_signed_with_payer(
            &ixs, Some(&payer.pubkey()), &[payer], svm.latest_blockhash(),
        );
        svm.send_transaction(tx).unwrap();

        // Open position
        svm.expire_blockhash();
        let (position, _) = position_pda(&info.pool, &payer.pubkey(), tick_lower, tick_upper);
        let ix = Instruction {
            program_id: kclmm_id(),
            accounts: kclmm::accounts::OpenPosition {
                payer: payer.pubkey(),
                owner: payer.pubkey(),
                pool: info.pool,
                position,
                system_program: system_program::ID,
            }.to_account_metas(None),
            data: kclmm::instruction::OpenPosition { tick_lower, tick_upper }.data(),
        };
        let tx = Transaction::new_signed_with_payer(
            &[compute_budget_ix(1_400_000), ix],
            Some(&payer.pubkey()), &[payer], svm.latest_blockhash(),
        );
        svm.send_transaction(tx).unwrap();

        // Add liquidity
        svm.expire_blockhash();
        let (tick_array_lower, _) = tick_array_pda(&info.pool, ta_lower_start);
        let (tick_array_upper, _) = tick_array_pda(&info.pool, ta_upper_start);
        let user_token_a = get_associated_token_address(&payer.pubkey(), &info.mint_a);
        let user_token_b = get_associated_token_address(&payer.pubkey(), &info.mint_b);

        let ix = Instruction {
            program_id: kclmm_id(),
            accounts: kclmm::accounts::AddLiquidity {
                owner: payer.pubkey(),
                pool: info.pool,
                position,
                tick_array_lower,
                tick_array_upper,
                vault_a: info.vault_a,
                vault_b: info.vault_b,
                user_token_a,
                user_token_b,
                token_program: spl_token::id(),
            }.to_account_metas(None),
            data: kclmm::instruction::AddLiquidity {
                liquidity_delta: liquidity,
                amount_a_max: u64::MAX,
                amount_b_max: u64::MAX,
            }.data(),
        };
        let tx = Transaction::new_signed_with_payer(
            &[compute_budget_ix(1_400_000), ix],
            Some(&payer.pubkey()), &[payer], svm.latest_blockhash(),
        );
        svm.send_transaction(tx).unwrap();
    }

    // ========== Environment ==========

    struct TestEnv {
        svm: LiteSVM,
        payer: Keypair,
        mint_authority: Keypair,
        mint_sol: Pubkey,
        mint_usdc: Pubkey,
        mint_eth: Pubkey,
        mint_btc: Pubkey,
        kpool_sol_usdc: KpoolInfo,
        kpool_usdc_eth: KpoolInfo,
        kpool_eth_btc: KpoolInfo,
        kclmm_sol_usdc: KclmmInfo,
        kclmm_usdc_eth: KclmmInfo,
    }

    fn setup() -> TestEnv {
        let mut svm = LiteSVM::new();
        let payer = Keypair::new();
        svm.airdrop(&payer.pubkey(), 100_000_000_000).unwrap();

        // Load programs
        svm.add_program(cpamm_id(), include_bytes!("../../../kpool/target/deploy/cpamm.so")).unwrap();
        svm.add_program(kclmm_id(), include_bytes!("../../../kclmm/target/deploy/kclmm.so")).unwrap();
        svm.add_program(kagg_id(), include_bytes!("../../target/deploy/kagg.so")).unwrap();

        let mint_authority = Keypair::new();
        svm.airdrop(&mint_authority.pubkey(), 1_000_000_000).unwrap();

        // Create 4 mints, canonically ordered
        let m1 = create_mint(&mut svm, &payer, &mint_authority.pubkey(), 6);
        let m2 = create_mint(&mut svm, &payer, &mint_authority.pubkey(), 6);
        let m3 = create_mint(&mut svm, &payer, &mint_authority.pubkey(), 6);
        let m4 = create_mint(&mut svm, &payer, &mint_authority.pubkey(), 6);
        let mut mints = [m1, m2, m3, m4];
        mints.sort();
        let [mint_sol, mint_usdc, mint_eth, mint_btc] = mints;

        // Create payer ATAs and fund with 10M tokens each
        for mint in &[mint_sol, mint_usdc, mint_eth, mint_btc] {
            let ata = create_ata(&mut svm, &payer, &payer.pubkey(), mint);
            mint_tokens(&mut svm, &payer, mint, &ata, &mint_authority, 10_000_000_000_000);
        }

        // === kpool SOL/USDC: 10k SOL + 1M USDC (price ~100) ===
        svm.expire_blockhash();
        let kpool_sol_usdc = init_cpamm_pool(&mut svm, &payer, &mint_sol, &mint_usdc);
        add_cpamm_liquidity(&mut svm, &payer, &kpool_sol_usdc, 10_000_000_000, 1_000_000_000_000);

        // === kpool USDC/ETH: 1M USDC + 500 ETH (price ~2000) ===
        svm.expire_blockhash();
        let kpool_usdc_eth = init_cpamm_pool(&mut svm, &payer, &mint_usdc, &mint_eth);
        add_cpamm_liquidity(&mut svm, &payer, &kpool_usdc_eth, 1_000_000_000_000, 500_000_000);

        // === kpool ETH/BTC: 500 ETH + 25 BTC (price ~20) ===
        svm.expire_blockhash();
        let kpool_eth_btc = init_cpamm_pool(&mut svm, &payer, &mint_eth, &mint_btc);
        add_cpamm_liquidity(&mut svm, &payer, &kpool_eth_btc, 500_000_000, 25_000_000);

        // === kclmm SOL/USDC: sqrt(100) = 10 * Q64, fee 0.30% ===
        svm.expire_blockhash();
        let mut kclmm_sol_usdc = init_kclmm_pool(
            &mut svm, &payer, &mint_sol, &mint_usdc,
            kclmm_c::FEE_RATE_30, 10u128 * kclmm_c::Q64,
        );
        init_tick_arrays_and_add_liquidity(
            &mut svm, &payer, &mut kclmm_sol_usdc,
            45060, 47040, 1_000_000_000u128,
        );

        // === kclmm USDC/ETH: price=1, sqrt_price=Q64, fee 0.30% ===
        svm.expire_blockhash();
        let mut kclmm_usdc_eth = init_kclmm_pool(
            &mut svm, &payer, &mint_usdc, &mint_eth,
            kclmm_c::FEE_RATE_30, kclmm_c::Q64,
        );
        init_tick_arrays_and_add_liquidity(
            &mut svm, &payer, &mut kclmm_usdc_eth,
            -3600, 3600, 1_000_000_000u128,
        );

        TestEnv {
            svm, payer, mint_authority,
            mint_sol, mint_usdc, mint_eth, mint_btc,
            kpool_sol_usdc, kpool_usdc_eth, kpool_eth_btc,
            kclmm_sol_usdc, kclmm_usdc_eth,
        }
    }

    // ========== kagg helpers ==========

    /// Build remaining_accounts for a kpool step (6 accounts)
    fn kpool_step_accounts(kpool: &KpoolInfo, input_mint: &Pubkey) -> Vec<AccountMeta> {
        vec![
            AccountMeta::new_readonly(cpamm_id(), false),
            AccountMeta::new(kpool.pool, false),
            AccountMeta::new_readonly(kpool.pool_authority, false),
            AccountMeta::new(kpool.vault_a, false),
            AccountMeta::new(kpool.vault_b, false),
            AccountMeta::new_readonly(*input_mint, false),
        ]
    }

    /// Build remaining_accounts for a kclmm step (6 + tick arrays)
    fn kclmm_step_accounts(clmm: &KclmmInfo, input_mint: &Pubkey) -> Vec<AccountMeta> {
        let mut accs = vec![
            AccountMeta::new_readonly(kclmm_id(), false),
            AccountMeta::new(clmm.pool, false),
            AccountMeta::new_readonly(clmm.pool_authority, false),
            AccountMeta::new(clmm.vault_a, false),
            AccountMeta::new(clmm.vault_b, false),
            AccountMeta::new_readonly(*input_mint, false),
        ];
        for &start in &clmm.tick_array_starts {
            let (ta, _) = tick_array_pda(&clmm.pool, start);
            accs.push(AccountMeta::new(ta, false));
        }
        accs
    }

    fn make_step(
        dex_id: kagg::types::DexId,
        num_accounts: u8,
        amount_in: u64,
        input_token_index: u8,
        output_token_index: u8,
        extra_data: Vec<u8>,
    ) -> kagg::types::RoutePlanStep {
        kagg::types::RoutePlanStep {
            dex_id, num_accounts, amount_in,
            input_token_index, output_token_index, extra_data,
        }
    }

    fn sqrt_limit_bytes(limit: u128) -> Vec<u8> {
        limit.to_le_bytes().to_vec()
    }

    /// Off-chain constant product quote with 0.3% fee
    fn quote_kpool(reserve_in: u64, reserve_out: u64, amount_in: u64) -> u64 {
        let amount_in_with_fee = amount_in as u128 * 9970;
        let numerator = reserve_out as u128 * amount_in_with_fee;
        let denominator = reserve_in as u128 * 10000 + amount_in_with_fee;
        (numerator / denominator) as u64
    }

    /// Execute a kagg route. Returns Ok(()) or error.
    fn exec_route(
        svm: &mut LiteSVM,
        payer: &Keypair,
        source_mint: Pubkey,
        dest_mint: Pubkey,
        route_plan: Vec<kagg::types::RoutePlanStep>,
        token_ledger: Vec<AccountMeta>,
        pool_accounts: Vec<AccountMeta>,
        minimum_amount_out: u64,
        cu_limit: u32,
    ) -> Result<(), litesvm::types::FailedTransactionMetadata> {
        let user_source = get_associated_token_address(&payer.pubkey(), &source_mint);
        let user_dest = get_associated_token_address(&payer.pubkey(), &dest_mint);

        let base_accounts = kagg::accounts::ExecuteRoute {
            user: payer.pubkey(),
            user_token_source: user_source,
            user_token_destination: user_dest,
            token_program: spl_token::id(),
        };

        let mut account_metas = base_accounts.to_account_metas(None);
        account_metas.extend(token_ledger.iter().cloned());
        account_metas.extend(pool_accounts.iter().cloned());

        let token_ledger_len = token_ledger.len() as u8;
        let ix_data = kagg::instruction::ExecuteRoute {
            route_plan,
            token_ledger_len,
            minimum_amount_out,
        };

        let ix = Instruction {
            program_id: kagg_id(),
            accounts: account_metas,
            data: ix_data.data(),
        };

        let tx = Transaction::new_signed_with_payer(
            &[compute_budget_ix(cu_limit), ix],
            Some(&payer.pubkey()),
            &[payer],
            svm.latest_blockhash(),
        );
        svm.send_transaction(tx)?;
        Ok(())
    }

    // ========== Tests ==========

    // Token index convention:
    //   0 = user_token_source
    //   1 = user_token_destination
    //   2+ = token_ledger[n-2]  (intermediate accounts)

    // ----- 1. Single-step swap through kpool -----
    #[test]
    fn test_01_single_step_kpool() {
        let mut env = setup();
        let amount_in = 1_000_000u64; // 1 token

        let step_accs = kpool_step_accounts(&env.kpool_sol_usdc, &env.mint_sol);
        let route_plan = vec![make_step(kagg::types::DexId::Kpool, 6, amount_in, 0, 1, vec![])];

        let dest = get_associated_token_address(&env.payer.pubkey(), &env.mint_usdc);
        let before = token_balance(&env.svm, &dest);

        exec_route(&mut env.svm, &env.payer, env.mint_sol, env.mint_usdc, route_plan, vec![], step_accs, 1, 400_000).unwrap();

        let after = token_balance(&env.svm, &dest);
        let output = after - before;
        let expected = quote_kpool(10_000_000_000, 1_000_000_000_000, amount_in);
        assert_eq!(output, expected, "Output should match constant-product formula");
    }

    // ----- 2. Single-step swap through kclmm -----
    #[test]
    fn test_02_single_step_kclmm() {
        let mut env = setup();
        let amount_in = 1_000_000u64;

        let step_accs = kclmm_step_accounts(&env.kclmm_sol_usdc, &env.mint_sol);
        let num = step_accs.len() as u8;
        let route_plan = vec![make_step(
            kagg::types::DexId::Kclmm, num, amount_in, 0, 1,
            sqrt_limit_bytes(kclmm_c::MIN_SQRT_PRICE + 1),
        )];

        let dest = get_associated_token_address(&env.payer.pubkey(), &env.mint_usdc);
        let before = token_balance(&env.svm, &dest);

        exec_route(&mut env.svm, &env.payer, env.mint_sol, env.mint_usdc, route_plan, vec![], step_accs, 1, 1_400_000).unwrap();

        let after = token_balance(&env.svm, &dest);
        assert!(after > before, "Should receive USDC from CLMM");
    }

    // ----- 3. 2-hop: SOL->USDC->ETH (kpool+kpool) -----
    #[test]
    fn test_03_two_hop_kpool_kpool() {
        let mut env = setup();
        let amount_in = 1_000_000u64;

        let user_usdc = get_associated_token_address(&env.payer.pubkey(), &env.mint_usdc);
        let token_ledger = vec![AccountMeta::new(user_usdc, false)];

        let step1 = kpool_step_accounts(&env.kpool_sol_usdc, &env.mint_sol);
        let step2 = kpool_step_accounts(&env.kpool_usdc_eth, &env.mint_usdc);
        let mut pool_accs = step1;
        pool_accs.extend(step2);

        let route_plan = vec![
            make_step(kagg::types::DexId::Kpool, 6, amount_in, 0, 2, vec![]),
            make_step(kagg::types::DexId::Kpool, 6, 0, 2, 1, vec![]),  // 0 = use prev output
        ];

        let dest = get_associated_token_address(&env.payer.pubkey(), &env.mint_eth);
        let before = token_balance(&env.svm, &dest);

        exec_route(&mut env.svm, &env.payer, env.mint_sol, env.mint_eth, route_plan, token_ledger, pool_accs, 1, 400_000).unwrap();

        let after = token_balance(&env.svm, &dest);
        assert!(after > before, "Should receive ETH through 2-hop kpool route");
    }

    // ----- 4. 2-hop: SOL->USDC->ETH (kpool+kclmm) -----
    #[test]
    fn test_04_two_hop_kpool_kclmm() {
        let mut env = setup();
        let amount_in = 1_000_000u64;

        let user_usdc = get_associated_token_address(&env.payer.pubkey(), &env.mint_usdc);
        let token_ledger = vec![AccountMeta::new(user_usdc, false)];

        let step1 = kpool_step_accounts(&env.kpool_sol_usdc, &env.mint_sol);
        let step2 = kclmm_step_accounts(&env.kclmm_usdc_eth, &env.mint_usdc);
        let step2_num = step2.len() as u8;
        let mut pool_accs = step1;
        pool_accs.extend(step2);

        let route_plan = vec![
            make_step(kagg::types::DexId::Kpool, 6, amount_in, 0, 2, vec![]),
            make_step(kagg::types::DexId::Kclmm, step2_num, 0, 2, 1,
                sqrt_limit_bytes(kclmm_c::MIN_SQRT_PRICE + 1)),
        ];

        let dest = get_associated_token_address(&env.payer.pubkey(), &env.mint_eth);
        let before = token_balance(&env.svm, &dest);

        exec_route(&mut env.svm, &env.payer, env.mint_sol, env.mint_eth, route_plan, token_ledger, pool_accs, 1, 1_400_000).unwrap();

        let after = token_balance(&env.svm, &dest);
        assert!(after > before, "Should receive ETH through kpool+kclmm route");
    }

    // ----- 5. 2-hop: SOL->USDC->ETH (kclmm+kpool) -----
    #[test]
    fn test_05_two_hop_kclmm_kpool() {
        let mut env = setup();
        let amount_in = 1_000_000u64;

        let user_usdc = get_associated_token_address(&env.payer.pubkey(), &env.mint_usdc);
        let token_ledger = vec![AccountMeta::new(user_usdc, false)];

        let step1 = kclmm_step_accounts(&env.kclmm_sol_usdc, &env.mint_sol);
        let step1_num = step1.len() as u8;
        let step2 = kpool_step_accounts(&env.kpool_usdc_eth, &env.mint_usdc);
        let mut pool_accs = step1;
        pool_accs.extend(step2);

        let route_plan = vec![
            make_step(kagg::types::DexId::Kclmm, step1_num, amount_in, 0, 2,
                sqrt_limit_bytes(kclmm_c::MIN_SQRT_PRICE + 1)),
            make_step(kagg::types::DexId::Kpool, 6, 0, 2, 1, vec![]),
        ];

        let dest = get_associated_token_address(&env.payer.pubkey(), &env.mint_eth);
        let before = token_balance(&env.svm, &dest);

        exec_route(&mut env.svm, &env.payer, env.mint_sol, env.mint_eth, route_plan, token_ledger, pool_accs, 1, 1_400_000).unwrap();

        let after = token_balance(&env.svm, &dest);
        assert!(after > before, "Should receive ETH through kclmm+kpool route");
    }

    // ----- 6. 3-hop: SOL->USDC->ETH->BTC -----
    #[test]
    fn test_06_three_hop() {
        let mut env = setup();
        let amount_in = 1_000_000u64;

        let user_usdc = get_associated_token_address(&env.payer.pubkey(), &env.mint_usdc);
        let user_eth = get_associated_token_address(&env.payer.pubkey(), &env.mint_eth);
        let token_ledger = vec![
            AccountMeta::new(user_usdc, false),  // index 2
            AccountMeta::new(user_eth, false),    // index 3
        ];

        let step1 = kpool_step_accounts(&env.kpool_sol_usdc, &env.mint_sol);
        let step2 = kpool_step_accounts(&env.kpool_usdc_eth, &env.mint_usdc);
        let step3 = kpool_step_accounts(&env.kpool_eth_btc, &env.mint_eth);
        let mut pool_accs = step1;
        pool_accs.extend(step2);
        pool_accs.extend(step3);

        let route_plan = vec![
            make_step(kagg::types::DexId::Kpool, 6, amount_in, 0, 2, vec![]),
            make_step(kagg::types::DexId::Kpool, 6, 0, 2, 3, vec![]),
            make_step(kagg::types::DexId::Kpool, 6, 0, 3, 1, vec![]),
        ];

        let dest = get_associated_token_address(&env.payer.pubkey(), &env.mint_btc);
        let before = token_balance(&env.svm, &dest);

        exec_route(&mut env.svm, &env.payer, env.mint_sol, env.mint_btc, route_plan, token_ledger, pool_accs, 1, 600_000).unwrap();

        let after = token_balance(&env.svm, &dest);
        assert!(after > before, "Should receive BTC through 3-hop route");
    }

    // ----- 7. Split: SOL->USDC via kpool(60%) + kclmm(40%) -----
    #[test]
    fn test_07_split_kpool_kclmm() {
        let mut env = setup();

        let step1 = kpool_step_accounts(&env.kpool_sol_usdc, &env.mint_sol);
        let step2 = kclmm_step_accounts(&env.kclmm_sol_usdc, &env.mint_sol);
        let step2_num = step2.len() as u8;
        let mut pool_accs = step1;
        pool_accs.extend(step2);

        // Both steps: source(0) -> destination(1), explicit amounts
        let route_plan = vec![
            make_step(kagg::types::DexId::Kpool, 6, 600_000, 0, 1, vec![]),
            make_step(kagg::types::DexId::Kclmm, step2_num, 400_000, 0, 1,
                sqrt_limit_bytes(kclmm_c::MIN_SQRT_PRICE + 1)),
        ];

        let dest = get_associated_token_address(&env.payer.pubkey(), &env.mint_usdc);
        let before = token_balance(&env.svm, &dest);

        exec_route(&mut env.svm, &env.payer, env.mint_sol, env.mint_usdc, route_plan, vec![], pool_accs, 1, 1_400_000).unwrap();

        let after = token_balance(&env.svm, &dest);
        assert!(after > before, "Should receive USDC from split route");
    }

    // ----- 8. 2-hop with explicit intermediate amount -----
    #[test]
    fn test_08_hop_explicit_amount() {
        let mut env = setup();

        let user_usdc = get_associated_token_address(&env.payer.pubkey(), &env.mint_usdc);
        let token_ledger = vec![AccountMeta::new(user_usdc, false)];

        let step1 = kpool_step_accounts(&env.kpool_sol_usdc, &env.mint_sol);
        let step2 = kpool_step_accounts(&env.kpool_usdc_eth, &env.mint_usdc);
        let mut pool_accs = step1;
        pool_accs.extend(step2);

        // Compute expected USDC output, use explicit amount for hop 2
        let usdc_out = quote_kpool(10_000_000_000, 1_000_000_000_000, 1_000_000);
        let route_plan = vec![
            make_step(kagg::types::DexId::Kpool, 6, 1_000_000, 0, 2, vec![]),
            make_step(kagg::types::DexId::Kpool, 6, usdc_out, 2, 1, vec![]),
        ];

        let dest = get_associated_token_address(&env.payer.pubkey(), &env.mint_eth);
        let before = token_balance(&env.svm, &dest);

        exec_route(&mut env.svm, &env.payer, env.mint_sol, env.mint_eth, route_plan, token_ledger, pool_accs, 1, 400_000).unwrap();

        let after = token_balance(&env.svm, &dest);
        assert!(after > before, "Should receive ETH through explicit-amount hop");
    }

    // ----- 9. Slippage pass (at exact minimum) -----
    #[test]
    fn test_09_slippage_pass() {
        let mut env = setup();
        let amount_in = 1_000_000u64;
        let expected = quote_kpool(10_000_000_000, 1_000_000_000_000, amount_in);

        let step_accs = kpool_step_accounts(&env.kpool_sol_usdc, &env.mint_sol);
        let route_plan = vec![make_step(kagg::types::DexId::Kpool, 6, amount_in, 0, 1, vec![])];

        exec_route(&mut env.svm, &env.payer, env.mint_sol, env.mint_usdc, route_plan, vec![], step_accs, expected, 400_000).unwrap();
    }

    // ----- 10. Slippage fail (below minimum) -----
    #[test]
    fn test_10_slippage_fail() {
        let mut env = setup();
        let amount_in = 1_000_000u64;
        let expected = quote_kpool(10_000_000_000, 1_000_000_000_000, amount_in);

        let step_accs = kpool_step_accounts(&env.kpool_sol_usdc, &env.mint_sol);
        let route_plan = vec![make_step(kagg::types::DexId::Kpool, 6, amount_in, 0, 1, vec![])];

        let result = exec_route(&mut env.svm, &env.payer, env.mint_sol, env.mint_usdc, route_plan, vec![], step_accs, expected + 1, 400_000);
        assert!(result.is_err(), "Should fail with slippage exceeded");
    }

    // ----- 11. Zero amount rejected -----
    #[test]
    fn test_11_zero_amount() {
        let mut env = setup();
        let step_accs = kpool_step_accounts(&env.kpool_sol_usdc, &env.mint_sol);
        let route_plan = vec![make_step(kagg::types::DexId::Kpool, 6, 0, 0, 1, vec![])];

        let result = exec_route(&mut env.svm, &env.payer, env.mint_sol, env.mint_usdc, route_plan, vec![], step_accs, 0, 400_000);
        assert!(result.is_err(), "Should reject zero amount on first step");
    }

    // ----- 12. Empty route plan rejected -----
    #[test]
    fn test_12_empty_route() {
        let mut env = setup();
        let result = exec_route(&mut env.svm, &env.payer, env.mint_sol, env.mint_usdc, vec![], vec![], vec![], 0, 400_000);
        assert!(result.is_err(), "Should reject empty route plan");
    }

    // ----- 13. Insufficient remaining_accounts -----
    #[test]
    fn test_13_insufficient_accounts() {
        let mut env = setup();
        let route_plan = vec![make_step(kagg::types::DexId::Kpool, 6, 1_000_000, 0, 1, vec![])];
        let partial = kpool_step_accounts(&env.kpool_sol_usdc, &env.mint_sol)[..3].to_vec();

        let result = exec_route(&mut env.svm, &env.payer, env.mint_sol, env.mint_usdc, route_plan, vec![], partial, 0, 400_000);
        assert!(result.is_err(), "Should reject insufficient accounts");
    }

    // ----- 14. Invalid token_ledger index -----
    #[test]
    fn test_14_invalid_ledger_index() {
        let mut env = setup();
        let step_accs = kpool_step_accounts(&env.kpool_sol_usdc, &env.mint_sol);
        // output_token_index=5 but no ledger entries
        let route_plan = vec![make_step(kagg::types::DexId::Kpool, 6, 1_000_000, 0, 5, vec![])];

        let result = exec_route(&mut env.svm, &env.payer, env.mint_sol, env.mint_usdc, route_plan, vec![], step_accs, 0, 400_000);
        assert!(result.is_err(), "Should reject invalid token ledger index");
    }

    // ----- 15. Output matches direct pool swap -----
    #[test]
    fn test_15_output_matches_direct() {
        let mut env = setup();
        let amount_in = 500_000u64;
        let expected = quote_kpool(10_000_000_000, 1_000_000_000_000, amount_in);

        let step_accs = kpool_step_accounts(&env.kpool_sol_usdc, &env.mint_sol);
        let route_plan = vec![make_step(kagg::types::DexId::Kpool, 6, amount_in, 0, 1, vec![])];

        let dest = get_associated_token_address(&env.payer.pubkey(), &env.mint_usdc);
        let before = token_balance(&env.svm, &dest);

        exec_route(&mut env.svm, &env.payer, env.mint_sol, env.mint_usdc, route_plan, vec![], step_accs, 1, 400_000).unwrap();

        let after = token_balance(&env.svm, &dest);
        assert_eq!(after - before, expected, "Kagg output should match direct pool swap exactly");
    }

    // ----- 16. 4-hop route within CU budget -----
    #[test]
    fn test_16_four_hop() {
        let mut env = setup();
        let amount_in = 1_000_000u64;

        // SOL -> USDC -> ETH -> BTC -> ETH (round trip last hop)
        let user_usdc = get_associated_token_address(&env.payer.pubkey(), &env.mint_usdc);
        let user_eth = get_associated_token_address(&env.payer.pubkey(), &env.mint_eth);
        let user_btc = get_associated_token_address(&env.payer.pubkey(), &env.mint_btc);

        let token_ledger = vec![
            AccountMeta::new(user_usdc, false),  // 2
            AccountMeta::new(user_eth, false),    // 3
            AccountMeta::new(user_btc, false),    // 4
        ];

        let step1 = kpool_step_accounts(&env.kpool_sol_usdc, &env.mint_sol);
        let step2 = kpool_step_accounts(&env.kpool_usdc_eth, &env.mint_usdc);
        let step3 = kpool_step_accounts(&env.kpool_eth_btc, &env.mint_eth);
        let step4 = kpool_step_accounts(&env.kpool_eth_btc, &env.mint_btc); // BTC->ETH reverse
        let mut pool_accs = step1;
        pool_accs.extend(step2);
        pool_accs.extend(step3);
        pool_accs.extend(step4);

        // Destination = ETH (index 1)
        let route_plan = vec![
            make_step(kagg::types::DexId::Kpool, 6, amount_in, 0, 2, vec![]),
            make_step(kagg::types::DexId::Kpool, 6, 0, 2, 3, vec![]),
            make_step(kagg::types::DexId::Kpool, 6, 0, 3, 4, vec![]),
            make_step(kagg::types::DexId::Kpool, 6, 0, 4, 1, vec![]),
        ];

        let dest = get_associated_token_address(&env.payer.pubkey(), &env.mint_eth);
        let before = token_balance(&env.svm, &dest);

        exec_route(&mut env.svm, &env.payer, env.mint_sol, env.mint_eth, route_plan, token_ledger, pool_accs, 1, 800_000).unwrap();

        let after = token_balance(&env.svm, &dest);
        assert!(after > before, "Should receive ETH through 4-hop route");
    }

    // ----- 17. Reverse direction swap (B->A) -----
    #[test]
    fn test_17_reverse_direction() {
        let mut env = setup();
        let amount_in = 100_000_000u64; // 100 USDC

        let step_accs = kpool_step_accounts(&env.kpool_sol_usdc, &env.mint_usdc);
        let route_plan = vec![make_step(kagg::types::DexId::Kpool, 6, amount_in, 0, 1, vec![])];

        let dest = get_associated_token_address(&env.payer.pubkey(), &env.mint_sol);
        let before = token_balance(&env.svm, &dest);

        exec_route(&mut env.svm, &env.payer, env.mint_usdc, env.mint_sol, route_plan, vec![], step_accs, 1, 400_000).unwrap();

        let after = token_balance(&env.svm, &dest);
        let output = after - before;
        let expected = quote_kpool(1_000_000_000_000, 10_000_000_000, amount_in);
        assert_eq!(output, expected, "Reverse direction should match formula");
    }

    // ----- 18. Multiple sequential routes (price impact) -----
    #[test]
    fn test_18_sequential_routes() {
        let mut env = setup();
        let amount_in = 500_000u64;

        let dest = get_associated_token_address(&env.payer.pubkey(), &env.mint_usdc);

        // First swap
        let step_accs = kpool_step_accounts(&env.kpool_sol_usdc, &env.mint_sol);
        let route_plan = vec![make_step(kagg::types::DexId::Kpool, 6, amount_in, 0, 1, vec![])];
        let before1 = token_balance(&env.svm, &dest);
        exec_route(&mut env.svm, &env.payer, env.mint_sol, env.mint_usdc, route_plan, vec![], step_accs, 1, 400_000).unwrap();
        let output1 = token_balance(&env.svm, &dest) - before1;
        env.svm.expire_blockhash();

        // Second swap — should get less due to price impact
        let step_accs = kpool_step_accounts(&env.kpool_sol_usdc, &env.mint_sol);
        let route_plan = vec![make_step(kagg::types::DexId::Kpool, 6, amount_in, 0, 1, vec![])];
        let before2 = token_balance(&env.svm, &dest);
        exec_route(&mut env.svm, &env.payer, env.mint_sol, env.mint_usdc, route_plan, vec![], step_accs, 1, 400_000).unwrap();
        let output2 = token_balance(&env.svm, &dest) - before2;

        assert!(output2 < output1, "Second swap should yield less: {} < {}", output2, output1);
    }

    // ----- 19. Quote accuracy multiple amounts -----
    #[test]
    fn test_19_quote_accuracy() {
        let mut env = setup();
        let dest = get_associated_token_address(&env.payer.pubkey(), &env.mint_usdc);

        // First amount should match exactly (pool at initial state)
        let amount = 100_000u64;
        let expected = quote_kpool(10_000_000_000, 1_000_000_000_000, amount);

        let step_accs = kpool_step_accounts(&env.kpool_sol_usdc, &env.mint_sol);
        let route_plan = vec![make_step(kagg::types::DexId::Kpool, 6, amount, 0, 1, vec![])];
        let before = token_balance(&env.svm, &dest);
        exec_route(&mut env.svm, &env.payer, env.mint_sol, env.mint_usdc, route_plan, vec![], step_accs, 1, 400_000).unwrap();
        let actual = token_balance(&env.svm, &dest) - before;
        assert_eq!(actual, expected, "First swap should exactly match off-chain quote");
    }

    // ----- 20. Full lifecycle -----
    #[test]
    fn test_20_full_lifecycle() {
        let mut env = setup();

        // 1. Single kpool: SOL -> USDC
        let step_accs = kpool_step_accounts(&env.kpool_sol_usdc, &env.mint_sol);
        let rp = vec![make_step(kagg::types::DexId::Kpool, 6, 1_000_000, 0, 1, vec![])];
        exec_route(&mut env.svm, &env.payer, env.mint_sol, env.mint_usdc, rp, vec![], step_accs, 1, 400_000).unwrap();
        env.svm.expire_blockhash();

        // 2. Single kclmm: SOL -> USDC
        let step_accs = kclmm_step_accounts(&env.kclmm_sol_usdc, &env.mint_sol);
        let n = step_accs.len() as u8;
        let rp = vec![make_step(kagg::types::DexId::Kclmm, n, 1_000_000, 0, 1,
            sqrt_limit_bytes(kclmm_c::MIN_SQRT_PRICE + 1))];
        exec_route(&mut env.svm, &env.payer, env.mint_sol, env.mint_usdc, rp, vec![], step_accs, 1, 1_400_000).unwrap();
        env.svm.expire_blockhash();

        // 3. 2-hop: SOL -> USDC -> ETH
        let user_usdc = get_associated_token_address(&env.payer.pubkey(), &env.mint_usdc);
        let tl = vec![AccountMeta::new(user_usdc, false)];
        let s1 = kpool_step_accounts(&env.kpool_sol_usdc, &env.mint_sol);
        let s2 = kpool_step_accounts(&env.kpool_usdc_eth, &env.mint_usdc);
        let mut pa = s1; pa.extend(s2);
        let rp = vec![
            make_step(kagg::types::DexId::Kpool, 6, 1_000_000, 0, 2, vec![]),
            make_step(kagg::types::DexId::Kpool, 6, 0, 2, 1, vec![]),
        ];
        exec_route(&mut env.svm, &env.payer, env.mint_sol, env.mint_eth, rp, tl, pa, 1, 400_000).unwrap();
        env.svm.expire_blockhash();

        // 4. 3-hop: SOL -> USDC -> ETH -> BTC
        let user_eth = get_associated_token_address(&env.payer.pubkey(), &env.mint_eth);
        let tl = vec![
            AccountMeta::new(user_usdc, false),
            AccountMeta::new(user_eth, false),
        ];
        let s1 = kpool_step_accounts(&env.kpool_sol_usdc, &env.mint_sol);
        let s2 = kpool_step_accounts(&env.kpool_usdc_eth, &env.mint_usdc);
        let s3 = kpool_step_accounts(&env.kpool_eth_btc, &env.mint_eth);
        let mut pa = s1; pa.extend(s2); pa.extend(s3);
        let rp = vec![
            make_step(kagg::types::DexId::Kpool, 6, 1_000_000, 0, 2, vec![]),
            make_step(kagg::types::DexId::Kpool, 6, 0, 2, 3, vec![]),
            make_step(kagg::types::DexId::Kpool, 6, 0, 3, 1, vec![]),
        ];
        let dest = get_associated_token_address(&env.payer.pubkey(), &env.mint_btc);
        let before = token_balance(&env.svm, &dest);
        exec_route(&mut env.svm, &env.payer, env.mint_sol, env.mint_btc, rp, tl, pa, 1, 600_000).unwrap();
        let after = token_balance(&env.svm, &dest);
        assert!(after > before, "Full lifecycle: 3-hop should produce BTC");
    }
}
