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
    const KROUTER_ID: &str = "hJ69REU7iZLsWzT1Bvw5w8Pe8Yz5kBR6dA42AczRj9Y";

    fn cpamm_id() -> Pubkey { Pubkey::from_str(CPAMM_ID).unwrap() }
    fn kclmm_id() -> Pubkey { Pubkey::from_str(KCLMM_ID).unwrap() }
    fn krouter_id() -> Pubkey { Pubkey::from_str(KROUTER_ID).unwrap() }

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
        fee_rate: u32,
        tick_spacing: u16,
        tick_array_starts: Vec<i32>,
    }

    // ========== Environment ==========

    struct TestEnv {
        svm: LiteSVM,
        payer: Keypair,
        mint_authority: Keypair,
        // 3 mints: SOL-like, USDC-like, ETH-like (all 6 decimals)
        mint_sol: Pubkey,
        mint_usdc: Pubkey,
        mint_eth: Pubkey,
        // kpool pools
        kpool_sol_usdc: KpoolInfo,
        kpool_usdc_eth: KpoolInfo,
        // kclmm pools
        kclmm_sol_usdc: KclmmInfo,
        kclmm_usdc_eth: KclmmInfo,
    }

    // ========== Setup functions ==========

    fn init_cpamm_pool(
        svm: &mut LiteSVM,
        payer: &Keypair,
        mint_a: &Pubkey,
        mint_b: &Pubkey,
    ) -> KpoolInfo {
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

    fn add_cpamm_liquidity(
        svm: &mut LiteSVM,
        payer: &Keypair,
        info: &KpoolInfo,
        amount_a: u64,
        amount_b: u64,
    ) {
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
        svm: &mut LiteSVM,
        payer: &Keypair,
        mint_a: &Pubkey,
        mint_b: &Pubkey,
        fee_rate: u32,
        initial_sqrt_price: u128,
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
            data: kclmm::instruction::InitPool {
                fee_rate,
                initial_sqrt_price,
            }.data(),
        };
        let tx = Transaction::new_signed_with_payer(
            &[compute_budget_ix(1_400_000), ix],
            Some(&payer.pubkey()), &[payer], svm.latest_blockhash(),
        );
        svm.send_transaction(tx).unwrap();

        KclmmInfo {
            pool, pool_authority, vault_a, vault_b,
            mint_a: *mint_a, mint_b: *mint_b,
            fee_rate, tick_spacing,
            tick_array_starts: vec![],
        }
    }

    fn init_tick_arrays_and_add_liquidity(
        svm: &mut LiteSVM,
        payer: &Keypair,
        info: &mut KclmmInfo,
        tick_lower: i32,
        tick_upper: i32,
        liquidity: u128,
    ) {
        // Init tick arrays
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

    fn setup() -> TestEnv {
        let mut svm = LiteSVM::new();
        let payer = Keypair::new();
        svm.airdrop(&payer.pubkey(), 100_000_000_000).unwrap();

        // Load all 3 programs
        svm.add_program(cpamm_id(), include_bytes!("../../../kpool/target/deploy/cpamm.so")).unwrap();
        svm.add_program(kclmm_id(), include_bytes!("../../../kclmm/target/deploy/kclmm.so")).unwrap();
        svm.add_program(krouter_id(), include_bytes!("../../target/deploy/krouter.so")).unwrap();

        // Mint authority
        let mint_authority = Keypair::new();
        svm.airdrop(&mint_authority.pubkey(), 1_000_000_000).unwrap();

        // Create 3 mints (SOL, USDC, ETH) — all 6 decimals
        // We need canonical ordering for each pair
        let m1 = create_mint(&mut svm, &payer, &mint_authority.pubkey(), 6);
        let m2 = create_mint(&mut svm, &payer, &mint_authority.pubkey(), 6);
        let m3 = create_mint(&mut svm, &payer, &mint_authority.pubkey(), 6);

        // Sort all three
        let mut mints = [m1, m2, m3];
        mints.sort();
        let [mint_sol, mint_usdc, mint_eth] = mints;

        // Create user ATAs and fund
        for mint in &[mint_sol, mint_usdc, mint_eth] {
            let ata = create_ata(&mut svm, &payer, &payer.pubkey(), mint);
            mint_tokens(&mut svm, &payer, mint, &ata, &mint_authority, 10_000_000_000_000); // 10M tokens
        }

        // === kpool SOL/USDC ===
        svm.expire_blockhash();
        let kpool_sol_usdc = init_cpamm_pool(&mut svm, &payer, &mint_sol, &mint_usdc);
        // Add liquidity: 10k SOL + 1M USDC (price ~100 USDC/SOL)
        add_cpamm_liquidity(&mut svm, &payer, &kpool_sol_usdc, 10_000_000_000, 1_000_000_000_000);

        // === kpool USDC/ETH ===
        svm.expire_blockhash();
        let kpool_usdc_eth = init_cpamm_pool(&mut svm, &payer, &mint_usdc, &mint_eth);
        // Add liquidity: 1M USDC + 500 ETH (price ~2000 USDC/ETH)
        add_cpamm_liquidity(&mut svm, &payer, &kpool_usdc_eth, 1_000_000_000_000, 500_000_000);

        // === kclmm SOL/USDC ===
        // sqrt(100) = 10, in Q64.64 = 10 * 2^64
        let sqrt_price_100 = 10u128 * kclmm_c::Q64;
        svm.expire_blockhash();
        let mut kclmm_sol_usdc = init_kclmm_pool(
            &mut svm, &payer, &mint_sol, &mint_usdc,
            kclmm_c::FEE_RATE_30, sqrt_price_100,
        );
        // Position range around tick ~46054 (price 100)
        let (tick_lower_su, tick_upper_su) = (45060i32, 47040i32);
        init_tick_arrays_and_add_liquidity(
            &mut svm, &payer, &mut kclmm_sol_usdc,
            tick_lower_su, tick_upper_su, 1_000_000_000u128,
        );

        // === kclmm USDC/ETH ===
        // Price: mint_usdc/mint_eth. In our sorted ordering, if usdc < eth:
        //   price = USDC per ETH = 2000, sqrt(2000) ≈ 44.72, Q64 = 44.72 * 2^64
        // If eth < usdc: price = ETH per USDC = 0.0005, sqrt ≈ 0.02236
        // We need to determine ordering. Since mints are sorted, mint_usdc < mint_eth.
        // So token_a = usdc, token_b = eth, price = amount_b_per_a = ETH/USDC...
        // Actually: for the pool, sqrt_price = sqrt(price of A in terms of B)
        // With USDC=A, ETH=B: if 1 USDC = 0.0005 ETH, sqrt(0.0005) ≈ 0.02236
        // Q64.64: 0.02236 * 2^64 ≈ 412_507_684_000_000_000 ≈ 4.125e17
        // Let's use a simpler price. Actually let's just use price=1 (1:1) for USDC/ETH
        // to keep the math simple. sqrt(1) = 1, Q64 = 2^64.
        let sqrt_price_1 = kclmm_c::Q64; // price = 1
        svm.expire_blockhash();
        let mut kclmm_usdc_eth = init_kclmm_pool(
            &mut svm, &payer, &mint_usdc, &mint_eth,
            kclmm_c::FEE_RATE_30, sqrt_price_1,
        );
        // Price=1, tick=0. Range: -3600..3600 (aligned to 60)
        let (tick_lower_ue, tick_upper_ue) = (-3600i32, 3600i32);
        init_tick_arrays_and_add_liquidity(
            &mut svm, &payer, &mut kclmm_usdc_eth,
            tick_lower_ue, tick_upper_ue, 1_000_000_000u128,
        );

        TestEnv {
            svm, payer, mint_authority,
            mint_sol, mint_usdc, mint_eth,
            kpool_sol_usdc, kpool_usdc_eth,
            kclmm_sol_usdc, kclmm_usdc_eth,
        }
    }

    // ========== Instruction builders ==========

    fn swap_kpool_ix(
        env: &TestEnv,
        kpool: &KpoolInfo,
        input_mint: &Pubkey,
        amount_in: u64,
        minimum_amount_out: u64,
    ) -> Instruction {
        let (user_token_in, user_token_out) = if *input_mint == kpool.mint_a {
            (
                get_associated_token_address(&env.payer.pubkey(), &kpool.mint_a),
                get_associated_token_address(&env.payer.pubkey(), &kpool.mint_b),
            )
        } else {
            (
                get_associated_token_address(&env.payer.pubkey(), &kpool.mint_b),
                get_associated_token_address(&env.payer.pubkey(), &kpool.mint_a),
            )
        };

        Instruction {
            program_id: krouter_id(),
            accounts: krouter::accounts::SwapKpool {
                user: env.payer.pubkey(),
                pool: kpool.pool,
                pool_authority: kpool.pool_authority,
                vault_a: kpool.vault_a,
                vault_b: kpool.vault_b,
                user_token_in,
                user_token_out,
                input_mint: *input_mint,
                cpamm_program: cpamm_id(),
                token_program: spl_token::id(),
            }.to_account_metas(None),
            data: krouter::instruction::SwapKpool {
                amount_in,
                minimum_amount_out,
            }.data(),
        }
    }

    fn swap_kclmm_ix(
        env: &TestEnv,
        clmm: &KclmmInfo,
        input_mint: &Pubkey,
        amount_in: u64,
        sqrt_price_limit: u128,
        minimum_amount_out: u64,
    ) -> Instruction {
        let a_to_b = *input_mint == clmm.mint_a;
        let (user_token_in, user_token_out) = if a_to_b {
            (
                get_associated_token_address(&env.payer.pubkey(), &clmm.mint_a),
                get_associated_token_address(&env.payer.pubkey(), &clmm.mint_b),
            )
        } else {
            (
                get_associated_token_address(&env.payer.pubkey(), &clmm.mint_b),
                get_associated_token_address(&env.payer.pubkey(), &clmm.mint_a),
            )
        };

        let mut accounts = krouter::accounts::SwapKclmm {
            user: env.payer.pubkey(),
            pool: clmm.pool,
            pool_authority: clmm.pool_authority,
            vault_a: clmm.vault_a,
            vault_b: clmm.vault_b,
            user_token_in,
            user_token_out,
            input_mint: *input_mint,
            kclmm_program: kclmm_id(),
            token_program: spl_token::id(),
        }.to_account_metas(None);

        // Add tick arrays as remaining_accounts
        for &start in &clmm.tick_array_starts {
            let (ta, _) = tick_array_pda(&clmm.pool, start);
            accounts.push(AccountMeta::new(ta, false));
        }

        Instruction {
            program_id: krouter_id(),
            accounts,
            data: krouter::instruction::SwapKclmm {
                amount_in,
                sqrt_price_limit,
                minimum_amount_out,
            }.data(),
        }
    }

    /// Build remaining_accounts for a kpool leg
    fn kpool_leg_accounts(kpool: &KpoolInfo, input_mint: &Pubkey) -> Vec<AccountMeta> {
        vec![
            AccountMeta::new_readonly(cpamm_id(), false),    // [0] dex program
            AccountMeta::new(kpool.pool, false),             // [1] pool
            AccountMeta::new_readonly(kpool.pool_authority, false), // [2] pool_authority
            AccountMeta::new(kpool.vault_a, false),          // [3] vault_a
            AccountMeta::new(kpool.vault_b, false),          // [4] vault_b
            AccountMeta::new_readonly(*input_mint, false),   // [5] input_mint
        ]
    }

    /// Build remaining_accounts for a kclmm leg
    fn kclmm_leg_accounts(clmm: &KclmmInfo, input_mint: &Pubkey) -> Vec<AccountMeta> {
        let mut accs = vec![
            AccountMeta::new_readonly(kclmm_id(), false),     // [0] dex program
            AccountMeta::new(clmm.pool, false),               // [1] pool
            AccountMeta::new_readonly(clmm.pool_authority, false), // [2] pool_authority
            AccountMeta::new(clmm.vault_a, false),            // [3] vault_a
            AccountMeta::new(clmm.vault_b, false),            // [4] vault_b
            AccountMeta::new_readonly(*input_mint, false),    // [5] input_mint
        ];
        // Add tick arrays
        for &start in &clmm.tick_array_starts {
            let (ta, _) = tick_array_pda(&clmm.pool, start);
            accs.push(AccountMeta::new(ta, false));
        }
        accs
    }

    fn route_two_hop_ix(
        env: &TestEnv,
        source_mint: &Pubkey,
        intermediate_mint: &Pubkey,
        destination_mint: &Pubkey,
        amount_in: u64,
        minimum_amount_out: u64,
        leg1: krouter::types::LegDescriptor,
        leg2: krouter::types::LegDescriptor,
        leg1_remaining: Vec<AccountMeta>,
        leg2_remaining: Vec<AccountMeta>,
    ) -> Instruction {
        let user_token_source = get_associated_token_address(&env.payer.pubkey(), source_mint);
        let user_token_intermediate = get_associated_token_address(&env.payer.pubkey(), intermediate_mint);
        let user_token_destination = get_associated_token_address(&env.payer.pubkey(), destination_mint);

        let mut accounts = krouter::accounts::RouteTwoHop {
            user: env.payer.pubkey(),
            user_token_source,
            user_token_intermediate,
            user_token_destination,
            token_program: spl_token::id(),
        }.to_account_metas(None);

        accounts.extend(leg1_remaining);
        accounts.extend(leg2_remaining);

        Instruction {
            program_id: krouter_id(),
            accounts,
            data: krouter::instruction::RouteTwoHop {
                amount_in,
                minimum_amount_out,
                leg1,
                leg2,
            }.data(),
        }
    }

    fn route_split_ix(
        env: &TestEnv,
        input_mint: &Pubkey,
        output_mint: &Pubkey,
        total_amount_in: u64,
        minimum_amount_out: u64,
        leg1: krouter::types::SplitLegDescriptor,
        leg2: krouter::types::SplitLegDescriptor,
        leg1_remaining: Vec<AccountMeta>,
        leg2_remaining: Vec<AccountMeta>,
    ) -> Instruction {
        let user_token_in = get_associated_token_address(&env.payer.pubkey(), input_mint);
        let user_token_out = get_associated_token_address(&env.payer.pubkey(), output_mint);

        let mut accounts = krouter::accounts::RouteSplit {
            user: env.payer.pubkey(),
            user_token_in,
            user_token_out,
            token_program: spl_token::id(),
        }.to_account_metas(None);

        accounts.extend(leg1_remaining);
        accounts.extend(leg2_remaining);

        Instruction {
            program_id: krouter_id(),
            accounts,
            data: krouter::instruction::RouteSplit {
                total_amount_in,
                minimum_amount_out,
                leg1,
                leg2,
            }.data(),
        }
    }

    // ========== Quote engine ==========

    /// Off-chain constant product quote with 0.3% fee
    fn quote_kpool(reserve_in: u64, reserve_out: u64, amount_in: u64) -> u64 {
        let amount_in_with_fee = amount_in as u128 * 9970; // 10000 - 30
        let numerator = reserve_out as u128 * amount_in_with_fee;
        let denominator = reserve_in as u128 * 10000 + amount_in_with_fee;
        (numerator / denominator) as u64
    }

    fn read_cpamm_pool(svm: &LiteSVM, pool: &Pubkey) -> (u64, u64) {
        let data = svm.get_account(pool).unwrap();
        let mut slice = &data.data[8..];
        let pool_state: cpamm::state::Pool = anchor_lang::AnchorDeserialize::deserialize(&mut slice).unwrap();
        (pool_state.reserve_a, pool_state.reserve_b)
    }

    // ========== TESTS ==========

    // --- Direct kpool swaps (tests 01-03) ---

    #[test]
    fn test_01_swap_kpool_a_to_b() {
        let mut env = setup();
        let amount_in = 1_000_000; // 1 SOL

        let (ra, rb) = read_cpamm_pool(&env.svm, &env.kpool_sol_usdc.pool);
        let expected_out = quote_kpool(ra, rb, amount_in);

        let user_out_before = token_balance(&env.svm,
            &get_associated_token_address(&env.payer.pubkey(), &env.mint_usdc));

        env.svm.expire_blockhash();
        let ix = swap_kpool_ix(&env, &env.kpool_sol_usdc, &env.mint_sol, amount_in, 1);
        let tx = Transaction::new_signed_with_payer(
            &[compute_budget_ix(400_000), ix],
            Some(&env.payer.pubkey()), &[&env.payer], env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        let user_out_after = token_balance(&env.svm,
            &get_associated_token_address(&env.payer.pubkey(), &env.mint_usdc));
        let actual_out = user_out_after - user_out_before;

        assert_eq!(actual_out, expected_out, "kpool A->B output mismatch");
        println!("kpool A->B: in={}, out={}", amount_in, actual_out);
    }

    #[test]
    fn test_02_swap_kpool_b_to_a() {
        let mut env = setup();
        let amount_in = 100_000_000; // 100 USDC

        let (ra, rb) = read_cpamm_pool(&env.svm, &env.kpool_sol_usdc.pool);
        let expected_out = quote_kpool(rb, ra, amount_in);

        let user_out_before = token_balance(&env.svm,
            &get_associated_token_address(&env.payer.pubkey(), &env.mint_sol));

        env.svm.expire_blockhash();
        let ix = swap_kpool_ix(&env, &env.kpool_sol_usdc, &env.mint_usdc, amount_in, 1);
        let tx = Transaction::new_signed_with_payer(
            &[compute_budget_ix(400_000), ix],
            Some(&env.payer.pubkey()), &[&env.payer], env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        let user_out_after = token_balance(&env.svm,
            &get_associated_token_address(&env.payer.pubkey(), &env.mint_sol));
        let actual_out = user_out_after - user_out_before;

        assert_eq!(actual_out, expected_out, "kpool B->A output mismatch");
        println!("kpool B->A: in={}, out={}", amount_in, actual_out);
    }

    #[test]
    fn test_03_swap_kpool_slippage_fail() {
        let mut env = setup();
        let amount_in = 1_000_000;

        env.svm.expire_blockhash();
        // Request impossibly high minimum
        let ix = swap_kpool_ix(&env, &env.kpool_sol_usdc, &env.mint_sol, amount_in, u64::MAX);
        let tx = Transaction::new_signed_with_payer(
            &[compute_budget_ix(400_000), ix],
            Some(&env.payer.pubkey()), &[&env.payer], env.svm.latest_blockhash(),
        );
        let result = env.svm.send_transaction(tx);
        assert!(result.is_err(), "Should fail with slippage exceeded");
        println!("kpool slippage fail: correctly rejected");
    }

    // --- Direct kclmm swaps (tests 04-06) ---

    #[test]
    fn test_04_swap_kclmm_a_to_b() {
        let mut env = setup();
        let amount_in = 1_000_000; // 1 SOL

        let user_out_before = token_balance(&env.svm,
            &get_associated_token_address(&env.payer.pubkey(), &env.mint_usdc));

        // For a_to_b, sqrt_price_limit < current price. Use MIN_SQRT_PRICE.
        let sqrt_price_limit = kclmm_c::MIN_SQRT_PRICE;

        env.svm.expire_blockhash();
        let ix = swap_kclmm_ix(&env, &env.kclmm_sol_usdc, &env.mint_sol, amount_in, sqrt_price_limit, 1);
        let tx = Transaction::new_signed_with_payer(
            &[compute_budget_ix(1_400_000), ix],
            Some(&env.payer.pubkey()), &[&env.payer], env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        let user_out_after = token_balance(&env.svm,
            &get_associated_token_address(&env.payer.pubkey(), &env.mint_usdc));
        let actual_out = user_out_after - user_out_before;

        assert!(actual_out > 0, "Should receive output tokens");
        println!("kclmm A->B: in={}, out={}", amount_in, actual_out);
    }

    #[test]
    fn test_05_swap_kclmm_b_to_a() {
        let mut env = setup();
        let amount_in = 100_000_000; // 100 USDC

        let user_out_before = token_balance(&env.svm,
            &get_associated_token_address(&env.payer.pubkey(), &env.mint_sol));

        // For b_to_a, sqrt_price_limit > current price. Use MAX_SQRT_PRICE.
        let sqrt_price_limit = kclmm_c::MAX_SQRT_PRICE;

        env.svm.expire_blockhash();
        let ix = swap_kclmm_ix(&env, &env.kclmm_sol_usdc, &env.mint_usdc, amount_in, sqrt_price_limit, 1);
        let tx = Transaction::new_signed_with_payer(
            &[compute_budget_ix(1_400_000), ix],
            Some(&env.payer.pubkey()), &[&env.payer], env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        let user_out_after = token_balance(&env.svm,
            &get_associated_token_address(&env.payer.pubkey(), &env.mint_sol));
        let actual_out = user_out_after - user_out_before;

        assert!(actual_out > 0, "Should receive output tokens");
        println!("kclmm B->A: in={}, out={}", amount_in, actual_out);
    }

    #[test]
    fn test_06_swap_kclmm_slippage_fail() {
        let mut env = setup();
        let amount_in = 1_000_000;
        let sqrt_price_limit = kclmm_c::MIN_SQRT_PRICE;

        env.svm.expire_blockhash();
        let ix = swap_kclmm_ix(&env, &env.kclmm_sol_usdc, &env.mint_sol, amount_in, sqrt_price_limit, u64::MAX);
        let tx = Transaction::new_signed_with_payer(
            &[compute_budget_ix(1_400_000), ix],
            Some(&env.payer.pubkey()), &[&env.payer], env.svm.latest_blockhash(),
        );
        let result = env.svm.send_transaction(tx);
        assert!(result.is_err(), "Should fail with slippage exceeded");
        println!("kclmm slippage fail: correctly rejected");
    }

    // --- Two-hop routes (tests 07-10) ---

    #[test]
    fn test_07_two_hop_kpool_kpool() {
        // SOL -> USDC -> ETH, both via kpool
        let mut env = setup();
        let amount_in = 1_000_000; // 1 SOL

        let user_eth_before = token_balance(&env.svm,
            &get_associated_token_address(&env.payer.pubkey(), &env.mint_eth));

        let leg1_accs = kpool_leg_accounts(&env.kpool_sol_usdc, &env.mint_sol);
        let leg2_accs = kpool_leg_accounts(&env.kpool_usdc_eth, &env.mint_usdc);

        let leg1 = krouter::types::LegDescriptor {
            pool_type: krouter::types::PoolType::Kpool,
            num_accounts: leg1_accs.len() as u8,
            sqrt_price_limit: 0,
        };
        let leg2 = krouter::types::LegDescriptor {
            pool_type: krouter::types::PoolType::Kpool,
            num_accounts: leg2_accs.len() as u8,
            sqrt_price_limit: 0,
        };

        env.svm.expire_blockhash();
        let ix = route_two_hop_ix(
            &env, &env.mint_sol, &env.mint_usdc, &env.mint_eth,
            amount_in, 1, leg1, leg2, leg1_accs, leg2_accs,
        );
        let tx = Transaction::new_signed_with_payer(
            &[compute_budget_ix(1_400_000), ix],
            Some(&env.payer.pubkey()), &[&env.payer], env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        let user_eth_after = token_balance(&env.svm,
            &get_associated_token_address(&env.payer.pubkey(), &env.mint_eth));
        let actual_out = user_eth_after - user_eth_before;

        assert!(actual_out > 0, "Should receive ETH");
        println!("two-hop kpool->kpool: in={} SOL, out={} ETH", amount_in, actual_out);
    }

    #[test]
    fn test_08_two_hop_kpool_kclmm() {
        // SOL -> USDC (kpool) -> ETH (kclmm)
        let mut env = setup();
        let amount_in = 1_000_000;

        let user_eth_before = token_balance(&env.svm,
            &get_associated_token_address(&env.payer.pubkey(), &env.mint_eth));

        let leg1_accs = kpool_leg_accounts(&env.kpool_sol_usdc, &env.mint_sol);
        let leg2_accs = kclmm_leg_accounts(&env.kclmm_usdc_eth, &env.mint_usdc);

        let leg1 = krouter::types::LegDescriptor {
            pool_type: krouter::types::PoolType::Kpool,
            num_accounts: leg1_accs.len() as u8,
            sqrt_price_limit: 0,
        };
        let leg2 = krouter::types::LegDescriptor {
            pool_type: krouter::types::PoolType::Kclmm,
            num_accounts: leg2_accs.len() as u8,
            sqrt_price_limit: kclmm_c::MIN_SQRT_PRICE, // USDC->ETH is a_to_b
        };

        env.svm.expire_blockhash();
        let ix = route_two_hop_ix(
            &env, &env.mint_sol, &env.mint_usdc, &env.mint_eth,
            amount_in, 1, leg1, leg2, leg1_accs, leg2_accs,
        );
        let tx = Transaction::new_signed_with_payer(
            &[compute_budget_ix(1_400_000), ix],
            Some(&env.payer.pubkey()), &[&env.payer], env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        let user_eth_after = token_balance(&env.svm,
            &get_associated_token_address(&env.payer.pubkey(), &env.mint_eth));
        let actual_out = user_eth_after - user_eth_before;

        assert!(actual_out > 0, "Should receive ETH");
        println!("two-hop kpool->kclmm: in={} SOL, out={} ETH", amount_in, actual_out);
    }

    #[test]
    fn test_09_two_hop_kclmm_kpool() {
        // SOL -> USDC (kclmm) -> ETH (kpool)
        let mut env = setup();
        let amount_in = 1_000_000;

        let user_eth_before = token_balance(&env.svm,
            &get_associated_token_address(&env.payer.pubkey(), &env.mint_eth));

        let leg1_accs = kclmm_leg_accounts(&env.kclmm_sol_usdc, &env.mint_sol);
        let leg2_accs = kpool_leg_accounts(&env.kpool_usdc_eth, &env.mint_usdc);

        let leg1 = krouter::types::LegDescriptor {
            pool_type: krouter::types::PoolType::Kclmm,
            num_accounts: leg1_accs.len() as u8,
            sqrt_price_limit: kclmm_c::MIN_SQRT_PRICE, // SOL->USDC is a_to_b
        };
        let leg2 = krouter::types::LegDescriptor {
            pool_type: krouter::types::PoolType::Kpool,
            num_accounts: leg2_accs.len() as u8,
            sqrt_price_limit: 0,
        };

        env.svm.expire_blockhash();
        let ix = route_two_hop_ix(
            &env, &env.mint_sol, &env.mint_usdc, &env.mint_eth,
            amount_in, 1, leg1, leg2, leg1_accs, leg2_accs,
        );
        let tx = Transaction::new_signed_with_payer(
            &[compute_budget_ix(1_400_000), ix],
            Some(&env.payer.pubkey()), &[&env.payer], env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        let user_eth_after = token_balance(&env.svm,
            &get_associated_token_address(&env.payer.pubkey(), &env.mint_eth));
        let actual_out = user_eth_after - user_eth_before;

        assert!(actual_out > 0, "Should receive ETH");
        println!("two-hop kclmm->kpool: in={} SOL, out={} ETH", amount_in, actual_out);
    }

    #[test]
    fn test_10_two_hop_slippage_fail() {
        let mut env = setup();
        let amount_in = 1_000_000;

        let leg1_accs = kpool_leg_accounts(&env.kpool_sol_usdc, &env.mint_sol);
        let leg2_accs = kpool_leg_accounts(&env.kpool_usdc_eth, &env.mint_usdc);

        let leg1 = krouter::types::LegDescriptor {
            pool_type: krouter::types::PoolType::Kpool,
            num_accounts: leg1_accs.len() as u8,
            sqrt_price_limit: 0,
        };
        let leg2 = krouter::types::LegDescriptor {
            pool_type: krouter::types::PoolType::Kpool,
            num_accounts: leg2_accs.len() as u8,
            sqrt_price_limit: 0,
        };

        env.svm.expire_blockhash();
        let ix = route_two_hop_ix(
            &env, &env.mint_sol, &env.mint_usdc, &env.mint_eth,
            amount_in, u64::MAX, leg1, leg2, leg1_accs, leg2_accs,
        );
        let tx = Transaction::new_signed_with_payer(
            &[compute_budget_ix(1_400_000), ix],
            Some(&env.payer.pubkey()), &[&env.payer], env.svm.latest_blockhash(),
        );
        let result = env.svm.send_transaction(tx);
        assert!(result.is_err(), "Should fail with slippage exceeded");
        println!("two-hop slippage fail: correctly rejected");
    }

    // --- Split routes (tests 11-12) ---

    #[test]
    fn test_11_split_kpool_kclmm() {
        // Split SOL -> USDC across kpool and kclmm
        let mut env = setup();
        let total = 2_000_000; // 2 SOL
        let leg1_amount = 1_000_000;
        let leg2_amount = 1_000_000;

        let user_usdc_before = token_balance(&env.svm,
            &get_associated_token_address(&env.payer.pubkey(), &env.mint_usdc));

        let leg1_accs = kpool_leg_accounts(&env.kpool_sol_usdc, &env.mint_sol);
        let leg2_accs = kclmm_leg_accounts(&env.kclmm_sol_usdc, &env.mint_sol);

        let leg1 = krouter::types::SplitLegDescriptor {
            pool_type: krouter::types::PoolType::Kpool,
            num_accounts: leg1_accs.len() as u8,
            amount_in: leg1_amount,
            sqrt_price_limit: 0,
        };
        let leg2 = krouter::types::SplitLegDescriptor {
            pool_type: krouter::types::PoolType::Kclmm,
            num_accounts: leg2_accs.len() as u8,
            amount_in: leg2_amount,
            sqrt_price_limit: kclmm_c::MIN_SQRT_PRICE,
        };

        env.svm.expire_blockhash();
        let ix = route_split_ix(
            &env, &env.mint_sol, &env.mint_usdc,
            total, 1, leg1, leg2, leg1_accs, leg2_accs,
        );
        let tx = Transaction::new_signed_with_payer(
            &[compute_budget_ix(1_400_000), ix],
            Some(&env.payer.pubkey()), &[&env.payer], env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        let user_usdc_after = token_balance(&env.svm,
            &get_associated_token_address(&env.payer.pubkey(), &env.mint_usdc));
        let actual_out = user_usdc_after - user_usdc_before;

        assert!(actual_out > 0, "Should receive USDC");
        println!("split kpool+kclmm: in={} SOL, out={} USDC", total, actual_out);
    }

    #[test]
    fn test_12_split_slippage_fail() {
        let mut env = setup();
        let total = 2_000_000;

        let leg1_accs = kpool_leg_accounts(&env.kpool_sol_usdc, &env.mint_sol);
        let leg2_accs = kclmm_leg_accounts(&env.kclmm_sol_usdc, &env.mint_sol);

        let leg1 = krouter::types::SplitLegDescriptor {
            pool_type: krouter::types::PoolType::Kpool,
            num_accounts: leg1_accs.len() as u8,
            amount_in: 1_000_000,
            sqrt_price_limit: 0,
        };
        let leg2 = krouter::types::SplitLegDescriptor {
            pool_type: krouter::types::PoolType::Kclmm,
            num_accounts: leg2_accs.len() as u8,
            amount_in: 1_000_000,
            sqrt_price_limit: kclmm_c::MIN_SQRT_PRICE,
        };

        env.svm.expire_blockhash();
        let ix = route_split_ix(
            &env, &env.mint_sol, &env.mint_usdc,
            total, u64::MAX, leg1, leg2, leg1_accs, leg2_accs,
        );
        let tx = Transaction::new_signed_with_payer(
            &[compute_budget_ix(1_400_000), ix],
            Some(&env.payer.pubkey()), &[&env.payer], env.svm.latest_blockhash(),
        );
        let result = env.svm.send_transaction(tx);
        assert!(result.is_err(), "Should fail with slippage exceeded");
        println!("split slippage fail: correctly rejected");
    }

    // --- Quote accuracy (tests 13-14) ---

    #[test]
    fn test_13_quote_accuracy_kpool() {
        let mut env = setup();
        let amount_in = 5_000_000; // 5 SOL

        let (ra, rb) = read_cpamm_pool(&env.svm, &env.kpool_sol_usdc.pool);
        let expected = quote_kpool(ra, rb, amount_in);

        let user_out_before = token_balance(&env.svm,
            &get_associated_token_address(&env.payer.pubkey(), &env.mint_usdc));

        env.svm.expire_blockhash();
        let ix = swap_kpool_ix(&env, &env.kpool_sol_usdc, &env.mint_sol, amount_in, 1);
        let tx = Transaction::new_signed_with_payer(
            &[compute_budget_ix(400_000), ix],
            Some(&env.payer.pubkey()), &[&env.payer], env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        let user_out_after = token_balance(&env.svm,
            &get_associated_token_address(&env.payer.pubkey(), &env.mint_usdc));
        let actual = user_out_after - user_out_before;

        assert_eq!(actual, expected, "Off-chain quote should match on-chain exactly");
        println!("kpool quote accuracy: expected={}, actual={}", expected, actual);
    }

    #[test]
    fn test_14_quote_accuracy_kclmm() {
        // For kclmm we just verify the output is in a reasonable range
        // (exact quoting requires full tick-stepping simulation)
        let mut env = setup();
        let amount_in = 1_000_000; // 1 SOL

        let user_out_before = token_balance(&env.svm,
            &get_associated_token_address(&env.payer.pubkey(), &env.mint_usdc));

        env.svm.expire_blockhash();
        let ix = swap_kclmm_ix(&env, &env.kclmm_sol_usdc, &env.mint_sol, amount_in,
            kclmm_c::MIN_SQRT_PRICE, 1);
        let tx = Transaction::new_signed_with_payer(
            &[compute_budget_ix(1_400_000), ix],
            Some(&env.payer.pubkey()), &[&env.payer], env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        let user_out_after = token_balance(&env.svm,
            &get_associated_token_address(&env.payer.pubkey(), &env.mint_usdc));
        let actual = user_out_after - user_out_before;

        // With price ~100 and 0.3% fee, 1 SOL should yield ~99.7 USDC
        // But concentrated liquidity is more efficient, so output should be in range [90, 110] USDC
        let min_expected = 90_000_000u64; // 90 USDC
        let max_expected = 110_000_000u64; // 110 USDC
        assert!(actual >= min_expected && actual <= max_expected,
            "kclmm output {} not in expected range [{}, {}]", actual, min_expected, max_expected);
        println!("kclmm quote accuracy: in=1 SOL, out={} USDC (~{:.2} USDC/SOL)",
            actual, actual as f64 / 1_000_000.0);
    }

    // --- Full lifecycle (test 15) ---

    #[test]
    fn test_15_full_lifecycle() {
        // Compare: direct kpool vs direct kclmm for same swap, verify router works for both
        let mut env = setup();
        let amount_in = 500_000; // 0.5 SOL

        // --- kpool path ---
        let (ra, rb) = read_cpamm_pool(&env.svm, &env.kpool_sol_usdc.pool);
        let kpool_expected = quote_kpool(ra, rb, amount_in);

        let user_usdc_before = token_balance(&env.svm,
            &get_associated_token_address(&env.payer.pubkey(), &env.mint_usdc));

        env.svm.expire_blockhash();
        let ix = swap_kpool_ix(&env, &env.kpool_sol_usdc, &env.mint_sol, amount_in, 1);
        let tx = Transaction::new_signed_with_payer(
            &[compute_budget_ix(400_000), ix],
            Some(&env.payer.pubkey()), &[&env.payer], env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        let user_usdc_mid = token_balance(&env.svm,
            &get_associated_token_address(&env.payer.pubkey(), &env.mint_usdc));
        let kpool_actual = user_usdc_mid - user_usdc_before;

        // --- kclmm path ---
        env.svm.expire_blockhash();
        let ix = swap_kclmm_ix(&env, &env.kclmm_sol_usdc, &env.mint_sol, amount_in,
            kclmm_c::MIN_SQRT_PRICE, 1);
        let tx = Transaction::new_signed_with_payer(
            &[compute_budget_ix(1_400_000), ix],
            Some(&env.payer.pubkey()), &[&env.payer], env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        let user_usdc_final = token_balance(&env.svm,
            &get_associated_token_address(&env.payer.pubkey(), &env.mint_usdc));
        let kclmm_actual = user_usdc_final - user_usdc_mid;

        println!("Full lifecycle comparison for 0.5 SOL -> USDC:");
        println!("  kpool:  {} USDC (quote: {})", kpool_actual, kpool_expected);
        println!("  kclmm:  {} USDC", kclmm_actual);

        // Both should give non-zero output
        assert!(kpool_actual > 0, "kpool should give output");
        assert!(kclmm_actual > 0, "kclmm should give output");

        // The better route gives more output
        let best = std::cmp::max(kpool_actual, kclmm_actual);
        let winner = if kclmm_actual >= kpool_actual { "kclmm" } else { "kpool" };
        println!("  Best route: {} ({} USDC)", winner, best);
    }
}
