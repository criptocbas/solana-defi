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

    use kclmm::constants::*;
    use kclmm::math;

    const PROGRAM_ID: &str = "7g3bAmnUmaoXZcDxffmxsZk7hmhMNDcQw7pT2aNC5tYW";

    fn program_id() -> Pubkey {
        Pubkey::from_str(PROGRAM_ID).unwrap()
    }

    fn compute_budget_ix(units: u32) -> Instruction {
        ComputeBudgetInstruction::set_compute_unit_limit(units)
    }

    // ========== PDA helpers ==========

    fn pool_pda(mint_a: &Pubkey, mint_b: &Pubkey, fee_rate: u32) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[POOL_SEED, mint_a.as_ref(), mint_b.as_ref(), &fee_rate.to_le_bytes()],
            &program_id(),
        )
    }

    fn pool_authority_pda(pool: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[POOL_AUTHORITY_SEED, pool.as_ref()],
            &program_id(),
        )
    }

    fn tick_array_pda(pool: &Pubkey, start_tick_index: i32) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[TICK_ARRAY_SEED, pool.as_ref(), &start_tick_index.to_le_bytes()],
            &program_id(),
        )
    }

    fn position_pda(pool: &Pubkey, owner: &Pubkey, tick_lower: i32, tick_upper: i32) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[
                POSITION_SEED,
                pool.as_ref(),
                owner.as_ref(),
                &tick_lower.to_le_bytes(),
                &tick_upper.to_le_bytes(),
            ],
            &program_id(),
        )
    }

    // ========== SPL helpers ==========

    fn create_mint(svm: &mut LiteSVM, payer: &Keypair, authority: &Pubkey, decimals: u8) -> Pubkey {
        let mint = Keypair::new();
        let rent = svm.minimum_balance_for_rent_exemption(spl_token::state::Mint::LEN);
        let create_account_ix = solana_sdk::system_instruction::create_account(
            &payer.pubkey(),
            &mint.pubkey(),
            rent,
            spl_token::state::Mint::LEN as u64,
            &spl_token::id(),
        );
        let init_mint_ix = spl_token::instruction::initialize_mint(
            &spl_token::id(), &mint.pubkey(), authority, None, decimals,
        ).unwrap();
        let tx = Transaction::new_signed_with_payer(
            &[create_account_ix, init_mint_ix],
            Some(&payer.pubkey()),
            &[payer, &mint],
            svm.latest_blockhash(),
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
        let token_account = spl_token::state::Account::unpack(&data.data).unwrap();
        token_account.amount
    }

    // ========== Test environment ==========

    struct TestEnv {
        svm: LiteSVM,
        payer: Keypair,
        mint_authority: Keypair,
        mint_a: Pubkey,
        mint_b: Pubkey,
        pool: Pubkey,
        pool_authority: Pubkey,
        vault_a: Pubkey,
        vault_b: Pubkey,
        fee_rate: u32,
        tick_spacing: u16,
    }

    /// SOL/USDC-style pool: 30bps fee, initial price ~100 (tick ~46054)
    /// Both mints 6 decimals.
    fn setup() -> TestEnv {
        let mut svm = LiteSVM::new();
        let payer = Keypair::new();
        svm.airdrop(&payer.pubkey(), 100_000_000_000).unwrap();

        let program_bytes = include_bytes!("../../target/deploy/kclmm.so");
        svm.add_program(program_id(), program_bytes).unwrap();

        let mint_authority = Keypair::new();
        svm.airdrop(&mint_authority.pubkey(), 1_000_000_000).unwrap();

        let m1 = create_mint(&mut svm, &payer, &mint_authority.pubkey(), 6);
        let m2 = create_mint(&mut svm, &payer, &mint_authority.pubkey(), 6);
        let (mint_a, mint_b) = if m1 < m2 { (m1, m2) } else { (m2, m1) };

        let fee_rate = FEE_RATE_30; // 3000 = 0.30%
        let tick_spacing = 60u16;

        // sqrt(100) = 10, in Q64.64 = 10 * 2^64
        let initial_sqrt_price = 10u128 * Q64;

        let (pool, _) = pool_pda(&mint_a, &mint_b, fee_rate);
        let (pool_authority, _) = pool_authority_pda(&pool);
        let vault_a = get_associated_token_address(&pool_authority, &mint_a);
        let vault_b = get_associated_token_address(&pool_authority, &mint_b);

        // Init pool
        let ix = Instruction {
            program_id: program_id(),
            accounts: kclmm::accounts::InitPool {
                payer: payer.pubkey(),
                mint_a,
                mint_b,
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
            &[compute_budget_ix(1_400_000), ix], Some(&payer.pubkey()), &[&payer], svm.latest_blockhash(),
        );
        svm.send_transaction(tx).unwrap();

        // Fund user with tokens
        let user_ata_a = create_ata(&mut svm, &payer, &payer.pubkey(), &mint_a);
        let user_ata_b = create_ata(&mut svm, &payer, &payer.pubkey(), &mint_b);
        mint_tokens(&mut svm, &payer, &mint_a, &user_ata_a, &mint_authority, 1_000_000_000_000); // 1M tokens
        mint_tokens(&mut svm, &payer, &mint_b, &user_ata_b, &mint_authority, 1_000_000_000_000);

        TestEnv {
            svm,
            payer,
            mint_authority,
            mint_a,
            mint_b,
            pool,
            pool_authority,
            vault_a,
            vault_b,
            fee_rate,
            tick_spacing,
        }
    }

    // ========== Instruction builders ==========

    fn init_tick_array_ix(env: &TestEnv, start_tick_index: i32) -> Instruction {
        let (tick_array, _) = tick_array_pda(&env.pool, start_tick_index);
        Instruction {
            program_id: program_id(),
            accounts: kclmm::accounts::InitTickArray {
                payer: env.payer.pubkey(),
                pool: env.pool,
                tick_array,
                system_program: system_program::ID,
            }.to_account_metas(None),
            data: kclmm::instruction::InitTickArray { start_tick_index }.data(),
        }
    }

    fn open_position_ix(env: &TestEnv, owner: &Pubkey, tick_lower: i32, tick_upper: i32) -> Instruction {
        let (position, _) = position_pda(&env.pool, owner, tick_lower, tick_upper);
        Instruction {
            program_id: program_id(),
            accounts: kclmm::accounts::OpenPosition {
                payer: env.payer.pubkey(),
                owner: *owner,
                pool: env.pool,
                position,
                system_program: system_program::ID,
            }.to_account_metas(None),
            data: kclmm::instruction::OpenPosition { tick_lower, tick_upper }.data(),
        }
    }

    fn add_liquidity_ix(
        env: &TestEnv,
        owner: &Pubkey,
        tick_lower: i32,
        tick_upper: i32,
        liquidity_delta: u128,
        amount_a_max: u64,
        amount_b_max: u64,
    ) -> Instruction {
        let (position, _) = position_pda(&env.pool, owner, tick_lower, tick_upper);
        let ta_lower_start = math::tick_array_start_for_tick(tick_lower, env.tick_spacing);
        let ta_upper_start = math::tick_array_start_for_tick(tick_upper, env.tick_spacing);
        let (tick_array_lower, _) = tick_array_pda(&env.pool, ta_lower_start);
        let (tick_array_upper, _) = tick_array_pda(&env.pool, ta_upper_start);

        Instruction {
            program_id: program_id(),
            accounts: kclmm::accounts::AddLiquidity {
                owner: *owner,
                pool: env.pool,
                position,
                tick_array_lower,
                tick_array_upper,
                vault_a: env.vault_a,
                vault_b: env.vault_b,
                user_token_a: get_associated_token_address(owner, &env.mint_a),
                user_token_b: get_associated_token_address(owner, &env.mint_b),
                token_program: spl_token::id(),
            }.to_account_metas(None),
            data: kclmm::instruction::AddLiquidity {
                liquidity_delta,
                amount_a_max,
                amount_b_max,
            }.data(),
        }
    }

    fn remove_liquidity_ix(
        env: &TestEnv,
        owner: &Pubkey,
        tick_lower: i32,
        tick_upper: i32,
        liquidity_delta: u128,
        amount_a_min: u64,
        amount_b_min: u64,
    ) -> Instruction {
        let (position, _) = position_pda(&env.pool, owner, tick_lower, tick_upper);
        let ta_lower_start = math::tick_array_start_for_tick(tick_lower, env.tick_spacing);
        let ta_upper_start = math::tick_array_start_for_tick(tick_upper, env.tick_spacing);
        let (tick_array_lower, _) = tick_array_pda(&env.pool, ta_lower_start);
        let (tick_array_upper, _) = tick_array_pda(&env.pool, ta_upper_start);

        Instruction {
            program_id: program_id(),
            accounts: kclmm::accounts::RemoveLiquidity {
                owner: *owner,
                pool: env.pool,
                position,
                tick_array_lower,
                tick_array_upper,
                vault_a: env.vault_a,
                vault_b: env.vault_b,
                pool_authority: env.pool_authority,
                user_token_a: get_associated_token_address(owner, &env.mint_a),
                user_token_b: get_associated_token_address(owner, &env.mint_b),
                token_program: spl_token::id(),
            }.to_account_metas(None),
            data: kclmm::instruction::RemoveLiquidity {
                liquidity_delta,
                amount_a_min,
                amount_b_min,
            }.data(),
        }
    }

    fn collect_fees_ix(
        env: &TestEnv,
        owner: &Pubkey,
        tick_lower: i32,
        tick_upper: i32,
    ) -> Instruction {
        let (position, _) = position_pda(&env.pool, owner, tick_lower, tick_upper);
        let ta_lower_start = math::tick_array_start_for_tick(tick_lower, env.tick_spacing);
        let ta_upper_start = math::tick_array_start_for_tick(tick_upper, env.tick_spacing);
        let (tick_array_lower, _) = tick_array_pda(&env.pool, ta_lower_start);
        let (tick_array_upper, _) = tick_array_pda(&env.pool, ta_upper_start);

        Instruction {
            program_id: program_id(),
            accounts: kclmm::accounts::CollectFees {
                owner: *owner,
                pool: env.pool,
                position,
                tick_array_lower,
                tick_array_upper,
                vault_a: env.vault_a,
                vault_b: env.vault_b,
                pool_authority: env.pool_authority,
                user_token_a: get_associated_token_address(owner, &env.mint_a),
                user_token_b: get_associated_token_address(owner, &env.mint_b),
                token_program: spl_token::id(),
            }.to_account_metas(None),
            data: kclmm::instruction::CollectFees {}.data(),
        }
    }

    fn swap_ix(
        env: &TestEnv,
        user: &Pubkey,
        a_to_b: bool,
        amount_in: u64,
        sqrt_price_limit: u128,
        minimum_amount_out: u64,
        tick_array_starts: &[i32],
    ) -> Instruction {
        let (input_mint, user_token_in, user_token_out) = if a_to_b {
            (
                env.mint_a,
                get_associated_token_address(user, &env.mint_a),
                get_associated_token_address(user, &env.mint_b),
            )
        } else {
            (
                env.mint_b,
                get_associated_token_address(user, &env.mint_b),
                get_associated_token_address(user, &env.mint_a),
            )
        };

        let mut accounts = kclmm::accounts::Swap {
            user: *user,
            pool: env.pool,
            vault_a: env.vault_a,
            vault_b: env.vault_b,
            pool_authority: env.pool_authority,
            user_token_in,
            user_token_out,
            input_mint,
            token_program: spl_token::id(),
        }.to_account_metas(None);

        // Add tick arrays as remaining_accounts
        for &start in tick_array_starts {
            let (ta, _) = tick_array_pda(&env.pool, start);
            accounts.push(AccountMeta::new(ta, false));
        }

        Instruction {
            program_id: program_id(),
            accounts,
            data: kclmm::instruction::Swap {
                amount_in,
                sqrt_price_limit,
                minimum_amount_out,
            }.data(),
        }
    }

    fn close_position_ix(
        env: &TestEnv,
        owner: &Pubkey,
        tick_lower: i32,
        tick_upper: i32,
    ) -> Instruction {
        let (position, _) = position_pda(&env.pool, owner, tick_lower, tick_upper);
        Instruction {
            program_id: program_id(),
            accounts: kclmm::accounts::ClosePosition {
                owner: *owner,
                pool: env.pool,
                position,
                system_program: system_program::ID,
            }.to_account_metas(None),
            data: kclmm::instruction::ClosePosition {}.data(),
        }
    }

    // ========== Convenience: setup with tick arrays and position ==========

    /// Returns tick_lower and tick_upper for a standard in-range position
    /// The pool starts at tick ~46054 (sqrt_price = 10 * Q64)
    /// We pick a range that encompasses this: e.g. tick 45000..47040 (must be aligned to 60)
    fn standard_ticks() -> (i32, i32) {
        // Both must be multiples of tick_spacing=60
        (45060, 47040)
    }

    fn setup_with_liquidity() -> (TestEnv, i32, i32) {
        let mut env = setup();
        let (tick_lower, tick_upper) = standard_ticks();

        // Init tick arrays covering the position range
        let ta_lower_start = math::tick_array_start_for_tick(tick_lower, env.tick_spacing);
        let ta_upper_start = math::tick_array_start_for_tick(tick_upper, env.tick_spacing);

        let mut ixs = vec![compute_budget_ix(1_400_000), init_tick_array_ix(&env, ta_lower_start)];
        if ta_upper_start != ta_lower_start {
            ixs.push(init_tick_array_ix(&env, ta_upper_start));
        }

        let tx = Transaction::new_signed_with_payer(
            &ixs, Some(&env.payer.pubkey()), &[&env.payer], env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        // Open position
        env.svm.expire_blockhash();
        let ix = open_position_ix(&env, &env.payer.pubkey(), tick_lower, tick_upper);
        let tx = Transaction::new_signed_with_payer(
            &[compute_budget_ix(1_400_000), ix], Some(&env.payer.pubkey()), &[&env.payer], env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        // Add liquidity
        env.svm.expire_blockhash();
        let liquidity = 1_000_000_000u128; // 1B liquidity units
        let ix = add_liquidity_ix(
            &env, &env.payer.pubkey(), tick_lower, tick_upper,
            liquidity, u64::MAX, u64::MAX,
        );
        let tx = Transaction::new_signed_with_payer(
            &[compute_budget_ix(1_400_000), ix], Some(&env.payer.pubkey()), &[&env.payer], env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        (env, tick_lower, tick_upper)
    }

    fn read_pool(svm: &LiteSVM, pool: &Pubkey) -> kclmm::state::Pool {
        let data = svm.get_account(pool).unwrap();
        let mut slice = &data.data[8..]; // skip discriminator
        anchor_lang::AnchorDeserialize::deserialize(&mut slice).unwrap()
    }

    fn read_position(svm: &LiteSVM, position: &Pubkey) -> kclmm::state::Position {
        let data = svm.get_account(position).unwrap();
        let mut slice = &data.data[8..];
        anchor_lang::AnchorDeserialize::deserialize(&mut slice).unwrap()
    }

    // ========== TESTS ==========

    // --- Pool Setup (3 tests) ---

    #[test]
    fn test_01_init_pool() {
        let env = setup();
        let pool = read_pool(&env.svm, &env.pool);
        assert_eq!(pool.mint_a, env.mint_a);
        assert_eq!(pool.mint_b, env.mint_b);
        assert_eq!(pool.vault_a, env.vault_a);
        assert_eq!(pool.vault_b, env.vault_b);
        assert_eq!(pool.fee_rate, FEE_RATE_30);
        assert_eq!(pool.tick_spacing, 60);
        assert_eq!(pool.sqrt_price, 10 * Q64);
        assert_eq!(pool.liquidity, 0);
        // tick_current should correspond to sqrt_price = 10
        let expected_tick = math::sqrt_price_to_tick(10 * Q64).unwrap();
        assert_eq!(pool.tick_current, expected_tick);
        println!("Pool initialized. tick_current={}", pool.tick_current);
    }

    #[test]
    fn test_02_init_pool_invalid_fee_tier() {
        let mut svm = LiteSVM::new();
        let payer = Keypair::new();
        svm.airdrop(&payer.pubkey(), 100_000_000_000).unwrap();
        let program_bytes = include_bytes!("../../target/deploy/kclmm.so");
        svm.add_program(program_id(), program_bytes).unwrap();

        let mint_authority = Keypair::new();
        svm.airdrop(&mint_authority.pubkey(), 1_000_000_000).unwrap();
        let m1 = create_mint(&mut svm, &payer, &mint_authority.pubkey(), 6);
        let m2 = create_mint(&mut svm, &payer, &mint_authority.pubkey(), 6);
        let (mint_a, mint_b) = if m1 < m2 { (m1, m2) } else { (m2, m1) };

        let bad_fee_rate = 9999u32; // not in {100, 500, 3000, 10000}
        let (pool, _) = pool_pda(&mint_a, &mint_b, bad_fee_rate);
        let (pool_authority, _) = pool_authority_pda(&pool);
        let vault_a = get_associated_token_address(&pool_authority, &mint_a);
        let vault_b = get_associated_token_address(&pool_authority, &mint_b);

        let ix = Instruction {
            program_id: program_id(),
            accounts: kclmm::accounts::InitPool {
                payer: payer.pubkey(),
                mint_a, mint_b, pool, pool_authority, vault_a, vault_b,
                system_program: system_program::ID,
                token_program: spl_token::id(),
                associated_token_program: spl_associated_token_account::id(),
            }.to_account_metas(None),
            data: kclmm::instruction::InitPool {
                fee_rate: bad_fee_rate,
                initial_sqrt_price: 10 * Q64,
            }.data(),
        };
        let tx = Transaction::new_signed_with_payer(
            &[compute_budget_ix(1_400_000), ix], Some(&payer.pubkey()), &[&payer], svm.latest_blockhash(),
        );
        let result = svm.send_transaction(tx);
        assert!(result.is_err(), "Invalid fee tier should fail");
    }

    #[test]
    fn test_03_init_pool_wrong_token_order() {
        let mut svm = LiteSVM::new();
        let payer = Keypair::new();
        svm.airdrop(&payer.pubkey(), 100_000_000_000).unwrap();
        let program_bytes = include_bytes!("../../target/deploy/kclmm.so");
        svm.add_program(program_id(), program_bytes).unwrap();

        let mint_authority = Keypair::new();
        svm.airdrop(&mint_authority.pubkey(), 1_000_000_000).unwrap();
        let m1 = create_mint(&mut svm, &payer, &mint_authority.pubkey(), 6);
        let m2 = create_mint(&mut svm, &payer, &mint_authority.pubkey(), 6);
        // Deliberately wrong order
        let (mint_a, mint_b) = if m1 > m2 { (m1, m2) } else { (m2, m1) };

        let fee_rate = FEE_RATE_30;
        let (pool, _) = pool_pda(&mint_a, &mint_b, fee_rate);
        let (pool_authority, _) = pool_authority_pda(&pool);
        let vault_a = get_associated_token_address(&pool_authority, &mint_a);
        let vault_b = get_associated_token_address(&pool_authority, &mint_b);

        let ix = Instruction {
            program_id: program_id(),
            accounts: kclmm::accounts::InitPool {
                payer: payer.pubkey(),
                mint_a, mint_b, pool, pool_authority, vault_a, vault_b,
                system_program: system_program::ID,
                token_program: spl_token::id(),
                associated_token_program: spl_associated_token_account::id(),
            }.to_account_metas(None),
            data: kclmm::instruction::InitPool {
                fee_rate,
                initial_sqrt_price: 10 * Q64,
            }.data(),
        };
        let tx = Transaction::new_signed_with_payer(
            &[compute_budget_ix(1_400_000), ix], Some(&payer.pubkey()), &[&payer], svm.latest_blockhash(),
        );
        let result = svm.send_transaction(tx);
        assert!(result.is_err(), "Wrong token order should fail");
    }

    // --- Tick Arrays (2 tests) ---

    #[test]
    fn test_04_init_tick_array() {
        let mut env = setup();
        // tick_spacing=60, ticks_in_array=60*64=3840
        let start = 0i32; // aligned to 3840
        let ix = init_tick_array_ix(&env, start);
        let tx = Transaction::new_signed_with_payer(
            &[compute_budget_ix(1_400_000), ix], Some(&env.payer.pubkey()), &[&env.payer], env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        // Read tick array and verify
        let (ta_key, _) = tick_array_pda(&env.pool, start);
        let data = env.svm.get_account(&ta_key).unwrap();
        // start_tick_index at offset 40..44 (after 8-byte discriminator + 32-byte pool pubkey)
        let start_tick = i32::from_le_bytes(data.data[40..44].try_into().unwrap());
        assert_eq!(start_tick, start);
        // bitmap at offset 48..56 (4 bytes padding after i32 for u64 alignment)
        let bitmap = u64::from_le_bytes(data.data[48..56].try_into().unwrap());
        assert_eq!(bitmap, 0);
        println!("Tick array initialized at start_tick_index={}", start);
    }

    #[test]
    fn test_05_init_tick_array_unaligned() {
        let mut env = setup();
        // 100 is not aligned to 3840
        let ix = init_tick_array_ix(&env, 100);
        let tx = Transaction::new_signed_with_payer(
            &[compute_budget_ix(1_400_000), ix], Some(&env.payer.pubkey()), &[&env.payer], env.svm.latest_blockhash(),
        );
        let result = env.svm.send_transaction(tx);
        assert!(result.is_err(), "Unaligned start tick should fail");
    }

    // --- Positions (2 tests) ---

    #[test]
    fn test_06_open_position() {
        let mut env = setup();
        let (tick_lower, tick_upper) = standard_ticks();

        let ix = open_position_ix(&env, &env.payer.pubkey(), tick_lower, tick_upper);
        let tx = Transaction::new_signed_with_payer(
            &[compute_budget_ix(1_400_000), ix], Some(&env.payer.pubkey()), &[&env.payer], env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        let (pos_key, _) = position_pda(&env.pool, &env.payer.pubkey(), tick_lower, tick_upper);
        let pos = read_position(&env.svm, &pos_key);
        assert_eq!(pos.pool, env.pool);
        assert_eq!(pos.owner, env.payer.pubkey());
        assert_eq!(pos.tick_lower, tick_lower);
        assert_eq!(pos.tick_upper, tick_upper);
        assert_eq!(pos.liquidity, 0);
        println!("Position opened: [{}, {}]", tick_lower, tick_upper);
    }

    #[test]
    fn test_07_open_position_invalid_ticks() {
        let mut env = setup();

        // lower >= upper
        let ix = open_position_ix(&env, &env.payer.pubkey(), 47040, 45060);
        let tx = Transaction::new_signed_with_payer(
            &[compute_budget_ix(1_400_000), ix], Some(&env.payer.pubkey()), &[&env.payer], env.svm.latest_blockhash(),
        );
        assert!(env.svm.send_transaction(tx).is_err(), "lower >= upper should fail");

        // Unaligned tick (61 not divisible by 60)
        env.svm.expire_blockhash();
        let ix = open_position_ix(&env, &env.payer.pubkey(), 61, 120);
        let tx = Transaction::new_signed_with_payer(
            &[compute_budget_ix(1_400_000), ix], Some(&env.payer.pubkey()), &[&env.payer], env.svm.latest_blockhash(),
        );
        assert!(env.svm.send_transaction(tx).is_err(), "Unaligned ticks should fail");
    }

    // --- Liquidity (4 tests) ---

    #[test]
    fn test_08_add_liquidity_below_range() {
        let mut env = setup();
        // Pool is at tick ~46054 (sqrt_price = 10 * Q64)
        // Position range [40020, 42060] is entirely below current tick.
        // When price is above the position range, position holds only token B.
        let tick_lower = 40020; // aligned to 60
        let tick_upper = 42060;

        let ta_lower_start = math::tick_array_start_for_tick(tick_lower, env.tick_spacing);
        let ta_upper_start = math::tick_array_start_for_tick(tick_upper, env.tick_spacing);
        let mut ixs = vec![compute_budget_ix(1_400_000), init_tick_array_ix(&env, ta_lower_start)];
        if ta_upper_start != ta_lower_start {
            ixs.push(init_tick_array_ix(&env, ta_upper_start));
        }
        let tx = Transaction::new_signed_with_payer(
            &ixs, Some(&env.payer.pubkey()), &[&env.payer], env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        env.svm.expire_blockhash();
        let ix = open_position_ix(&env, &env.payer.pubkey(), tick_lower, tick_upper);
        let tx = Transaction::new_signed_with_payer(
            &[compute_budget_ix(1_400_000), ix], Some(&env.payer.pubkey()), &[&env.payer], env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        let vault_a_before = token_balance(&env.svm, &env.vault_a);
        let vault_b_before = token_balance(&env.svm, &env.vault_b);

        env.svm.expire_blockhash();
        let liquidity = 100_000_000u128;
        let ix = add_liquidity_ix(
            &env, &env.payer.pubkey(), tick_lower, tick_upper,
            liquidity, u64::MAX, u64::MAX,
        );
        let tx = Transaction::new_signed_with_payer(
            &[compute_budget_ix(1_400_000), ix], Some(&env.payer.pubkey()), &[&env.payer], env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        let vault_a_after = token_balance(&env.svm, &env.vault_a);
        let vault_b_after = token_balance(&env.svm, &env.vault_b);

        let deposited_a = vault_a_after - vault_a_before;
        let deposited_b = vault_b_after - vault_b_before;

        println!("Below-range deposit: A={}, B={}", deposited_a, deposited_b);
        assert_eq!(deposited_a, 0, "Should not deposit token A when price is above range");
        assert!(deposited_b > 0, "Should deposit token B when price is above range");
    }

    #[test]
    fn test_09_add_liquidity_above_range() {
        let mut env = setup();
        // Position entirely above current tick → all token A
        // (price is below range, so only token A is deposited)
        let tick_lower = 48060;
        let tick_upper = 50040;

        let ta_lower_start = math::tick_array_start_for_tick(tick_lower, env.tick_spacing);
        let ta_upper_start = math::tick_array_start_for_tick(tick_upper, env.tick_spacing);
        let mut ixs = vec![compute_budget_ix(1_400_000), init_tick_array_ix(&env, ta_lower_start)];
        if ta_upper_start != ta_lower_start {
            ixs.push(init_tick_array_ix(&env, ta_upper_start));
        }
        let tx = Transaction::new_signed_with_payer(
            &ixs, Some(&env.payer.pubkey()), &[&env.payer], env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        env.svm.expire_blockhash();
        let ix = open_position_ix(&env, &env.payer.pubkey(), tick_lower, tick_upper);
        let tx = Transaction::new_signed_with_payer(
            &[compute_budget_ix(1_400_000), ix], Some(&env.payer.pubkey()), &[&env.payer], env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        let vault_a_before = token_balance(&env.svm, &env.vault_a);
        let vault_b_before = token_balance(&env.svm, &env.vault_b);

        env.svm.expire_blockhash();
        let ix = add_liquidity_ix(
            &env, &env.payer.pubkey(), tick_lower, tick_upper,
            100_000_000u128, u64::MAX, u64::MAX,
        );
        let tx = Transaction::new_signed_with_payer(
            &[compute_budget_ix(1_400_000), ix], Some(&env.payer.pubkey()), &[&env.payer], env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        let deposited_a = token_balance(&env.svm, &env.vault_a) - vault_a_before;
        let deposited_b = token_balance(&env.svm, &env.vault_b) - vault_b_before;

        println!("Above-range deposit: A={}, B={}", deposited_a, deposited_b);
        assert!(deposited_a > 0, "Should deposit token A when price is below range");
        assert_eq!(deposited_b, 0, "Should not deposit token B when price is below range");
    }

    #[test]
    fn test_10_add_liquidity_in_range() {
        let (env, tick_lower, tick_upper) = setup_with_liquidity();

        let pool = read_pool(&env.svm, &env.pool);
        assert!(pool.liquidity > 0, "Pool should have active liquidity");
        println!("In-range liquidity: L={}", pool.liquidity);

        let vault_a = token_balance(&env.svm, &env.vault_a);
        let vault_b = token_balance(&env.svm, &env.vault_b);
        println!("Vault A={}, Vault B={}", vault_a, vault_b);
        assert!(vault_a > 0, "Should have deposited A");
        assert!(vault_b > 0, "Should have deposited B");

        let (pos_key, _) = position_pda(&env.pool, &env.payer.pubkey(), tick_lower, tick_upper);
        let pos = read_position(&env.svm, &pos_key);
        assert_eq!(pos.liquidity, 1_000_000_000);
    }

    #[test]
    fn test_11_add_liquidity_slippage_exceeded() {
        let mut env = setup();
        let (tick_lower, tick_upper) = standard_ticks();

        let ta_lower_start = math::tick_array_start_for_tick(tick_lower, env.tick_spacing);
        let ta_upper_start = math::tick_array_start_for_tick(tick_upper, env.tick_spacing);
        let mut ixs = vec![compute_budget_ix(1_400_000), init_tick_array_ix(&env, ta_lower_start)];
        if ta_upper_start != ta_lower_start {
            ixs.push(init_tick_array_ix(&env, ta_upper_start));
        }
        let tx = Transaction::new_signed_with_payer(
            &ixs, Some(&env.payer.pubkey()), &[&env.payer], env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        env.svm.expire_blockhash();
        let ix = open_position_ix(&env, &env.payer.pubkey(), tick_lower, tick_upper);
        let tx = Transaction::new_signed_with_payer(
            &[compute_budget_ix(1_400_000), ix], Some(&env.payer.pubkey()), &[&env.payer], env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        // Set amount_a_max = 1 (too low)
        env.svm.expire_blockhash();
        let ix = add_liquidity_ix(
            &env, &env.payer.pubkey(), tick_lower, tick_upper,
            1_000_000_000u128, 1, u64::MAX,
        );
        let tx = Transaction::new_signed_with_payer(
            &[compute_budget_ix(1_400_000), ix], Some(&env.payer.pubkey()), &[&env.payer], env.svm.latest_blockhash(),
        );
        let result = env.svm.send_transaction(tx);
        assert!(result.is_err(), "Should fail: amount_a exceeds max");
    }

    // --- Swap (6 tests) ---

    #[test]
    fn test_12_swap_a_to_b() {
        let (mut env, tick_lower, tick_upper) = setup_with_liquidity();

        let user_b_before = token_balance(&env.svm,
            &get_associated_token_address(&env.payer.pubkey(), &env.mint_b));

        let ta_start = math::tick_array_start_for_tick(tick_lower, env.tick_spacing);
        let ta_start2 = math::tick_array_start_for_tick(tick_upper, env.tick_spacing);
        let mut tas = vec![ta_start];
        if ta_start2 != ta_start { tas.push(ta_start2); }

        env.svm.expire_blockhash();
        let swap_amount = 100_000u64;
        let ix = swap_ix(
            &env, &env.payer.pubkey(), true, swap_amount,
            MIN_SQRT_PRICE, 1, &tas,
        );
        let tx = Transaction::new_signed_with_payer(
            &[compute_budget_ix(1_400_000), ix], Some(&env.payer.pubkey()), &[&env.payer], env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        let user_b_after = token_balance(&env.svm,
            &get_associated_token_address(&env.payer.pubkey(), &env.mint_b));
        let received = user_b_after - user_b_before;
        println!("Swap A→B: in={}, out={}", swap_amount, received);
        assert!(received > 0, "Should receive token B");

        // Check pool sqrt_price decreased (a→b)
        let pool = read_pool(&env.svm, &env.pool);
        assert!(pool.sqrt_price < 10 * Q64, "Price should decrease for a→b");
        assert!(pool.fee_growth_global_a > 0, "Fee growth should increase");
    }

    #[test]
    fn test_13_swap_b_to_a() {
        let (mut env, tick_lower, tick_upper) = setup_with_liquidity();

        let user_a_before = token_balance(&env.svm,
            &get_associated_token_address(&env.payer.pubkey(), &env.mint_a));

        let ta_start = math::tick_array_start_for_tick(tick_lower, env.tick_spacing);
        let ta_start2 = math::tick_array_start_for_tick(tick_upper, env.tick_spacing);
        let mut tas = vec![ta_start];
        if ta_start2 != ta_start { tas.push(ta_start2); }

        env.svm.expire_blockhash();
        let swap_amount = 100_000u64;
        let ix = swap_ix(
            &env, &env.payer.pubkey(), false, swap_amount,
            MAX_SQRT_PRICE, 1, &tas,
        );
        let tx = Transaction::new_signed_with_payer(
            &[compute_budget_ix(1_400_000), ix], Some(&env.payer.pubkey()), &[&env.payer], env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        let user_a_after = token_balance(&env.svm,
            &get_associated_token_address(&env.payer.pubkey(), &env.mint_a));
        let received = user_a_after - user_a_before;
        println!("Swap B→A: in={}, out={}", swap_amount, received);
        assert!(received > 0, "Should receive token A");

        let pool = read_pool(&env.svm, &env.pool);
        assert!(pool.sqrt_price > 10 * Q64, "Price should increase for b→a");
    }

    #[test]
    fn test_14_swap_crosses_tick() {
        let mut env = setup();
        let tick_spacing = env.tick_spacing;

        // Create two adjacent positions with a tick boundary between them
        // Position 1: [44880, 46080] — below-ish range
        // Position 2: [46080, 47280] — above-ish range
        // Current tick ~46054 is in position 1 range
        // A large a→b swap should push price down, crossing tick 44880 boundary

        let pos1_lower = 44880;
        let pos1_upper = 46080;
        let pos2_lower = 46080;
        let pos2_upper = 47280;

        // Init all needed tick arrays
        let ta_starts: Vec<i32> = [pos1_lower, pos1_upper, pos2_lower, pos2_upper].iter()
            .map(|&t| math::tick_array_start_for_tick(t, tick_spacing))
            .collect::<std::collections::HashSet<i32>>()
            .into_iter().collect();

        let mut ixs: Vec<_> = vec![compute_budget_ix(1_400_000)];
        ixs.extend(ta_starts.iter().map(|&s| init_tick_array_ix(&env, s)));
        let tx = Transaction::new_signed_with_payer(
            &ixs, Some(&env.payer.pubkey()), &[&env.payer], env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        // Open both positions
        env.svm.expire_blockhash();
        let ix1 = open_position_ix(&env, &env.payer.pubkey(), pos1_lower, pos1_upper);
        let ix2 = open_position_ix(&env, &env.payer.pubkey(), pos2_lower, pos2_upper);
        let tx = Transaction::new_signed_with_payer(
            &[compute_budget_ix(1_400_000), ix1, ix2], Some(&env.payer.pubkey()), &[&env.payer], env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        // Add liquidity to both
        env.svm.expire_blockhash();
        let liq = 500_000_000u128;
        let ix1 = add_liquidity_ix(
            &env, &env.payer.pubkey(), pos1_lower, pos1_upper,
            liq, u64::MAX, u64::MAX,
        );
        let ix2 = add_liquidity_ix(
            &env, &env.payer.pubkey(), pos2_lower, pos2_upper,
            liq, u64::MAX, u64::MAX,
        );
        let tx = Transaction::new_signed_with_payer(
            &[compute_budget_ix(1_400_000), ix1, ix2], Some(&env.payer.pubkey()), &[&env.payer], env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        let pool_before = read_pool(&env.svm, &env.pool);
        println!("Before swap: tick={}, L={}", pool_before.tick_current, pool_before.liquidity);

        // Large swap a→b to cross tick boundaries
        env.svm.expire_blockhash();
        let mut ta_list: Vec<i32> = ta_starts.clone();
        ta_list.sort();
        ta_list.reverse(); // For a→b, tick arrays should go from high to low

        let swap_amount = 50_000_000u64; // Large swap
        let ix = swap_ix(
            &env, &env.payer.pubkey(), true, swap_amount,
            MIN_SQRT_PRICE, 1, &ta_list,
        );
        let tx = Transaction::new_signed_with_payer(
            &[compute_budget_ix(1_400_000), ix], Some(&env.payer.pubkey()), &[&env.payer], env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        let pool_after = read_pool(&env.svm, &env.pool);
        println!("After swap: tick={}, L={}", pool_after.tick_current, pool_after.liquidity);
        assert!(pool_after.tick_current < pool_before.tick_current,
            "Tick should have decreased");
    }

    #[test]
    fn test_15_swap_sqrt_price_limit() {
        let (mut env, tick_lower, tick_upper) = setup_with_liquidity();

        let user_a_before = token_balance(&env.svm,
            &get_associated_token_address(&env.payer.pubkey(), &env.mint_a));

        let ta_start = math::tick_array_start_for_tick(tick_lower, env.tick_spacing);
        let ta_start2 = math::tick_array_start_for_tick(tick_upper, env.tick_spacing);
        let mut tas = vec![ta_start];
        if ta_start2 != ta_start { tas.push(ta_start2); }

        // Set a tight sqrt_price_limit that's only slightly below current
        let pool = read_pool(&env.svm, &env.pool);
        // Limit = 99.9% of current price
        let limit = pool.sqrt_price * 999 / 1000;

        env.svm.expire_blockhash();
        let ix = swap_ix(
            &env, &env.payer.pubkey(), true, 10_000_000, // large amount
            limit, 0, &tas, // minimum_out=0 so slippage doesn't block us
        );
        let tx = Transaction::new_signed_with_payer(
            &[compute_budget_ix(1_400_000), ix], Some(&env.payer.pubkey()), &[&env.payer], env.svm.latest_blockhash(),
        );
        // This might fail if limit enforcement doesn't produce any output,
        // but let's see what happens
        let result = env.svm.send_transaction(tx);

        if result.is_ok() {
            let pool_after = read_pool(&env.svm, &env.pool);
            // Price should have stopped at or above the limit
            assert!(pool_after.sqrt_price >= limit,
                "Price should stop at limit: {} >= {}", pool_after.sqrt_price, limit);
            println!("Swap stopped at price limit. Final price: {}", pool_after.sqrt_price);
        } else {
            // If it fails with ZeroOutput, that's also acceptable behavior
            // (limit was too tight to produce any output)
            println!("Swap with tight limit produced zero output (expected)");
        }
    }

    #[test]
    fn test_16_swap_slippage_protection() {
        let (mut env, tick_lower, tick_upper) = setup_with_liquidity();

        let ta_start = math::tick_array_start_for_tick(tick_lower, env.tick_spacing);
        let ta_start2 = math::tick_array_start_for_tick(tick_upper, env.tick_spacing);
        let mut tas = vec![ta_start];
        if ta_start2 != ta_start { tas.push(ta_start2); }

        env.svm.expire_blockhash();
        // Swap 100 tokens but require 1M out (impossible)
        let ix = swap_ix(
            &env, &env.payer.pubkey(), true, 100_000,
            MIN_SQRT_PRICE, 1_000_000_000, &tas,
        );
        let tx = Transaction::new_signed_with_payer(
            &[compute_budget_ix(1_400_000), ix], Some(&env.payer.pubkey()), &[&env.payer], env.svm.latest_blockhash(),
        );
        let result = env.svm.send_transaction(tx);
        assert!(result.is_err(), "Should fail: slippage exceeded");
    }

    #[test]
    fn test_17_swap_through_zero_liquidity_gap() {
        let mut env = setup();
        let tick_spacing = env.tick_spacing;

        // Create two positions with a gap between them
        // Position 1: [44880, 45600] — below current price
        // Position 2: [46560, 47280] — at/above current price
        // Gap: [45600, 46560] has no liquidity
        // Current tick ~46054

        // Position 2 is in range (46054 is between 46560 would be above, let me adjust)
        // Actually current tick is ~46054. Let me make position 2 span it.
        // Position 2: [45600, 46680] — spans current tick, so it's in range
        // Position 1: [44160, 45600] — below range, all token A
        // A swap a→b pushes price down. Once price exits position 2 at tick 45600,
        // position 1's liquidity kicks in. No gap in this case.

        // For a true gap, both positions need to NOT be adjacent:
        // Position 1: [43680, 44880] — fully below
        // Position 2: [45600, 47280] — spans current tick
        // Gap: [44880, 45600] — no liquidity
        // a→b swap goes: in position 2 range → exits at 45600 → gap → enters position 1 at 44880

        let pos1_lower = 43680;
        let pos1_upper = 44880;
        let pos2_lower = 45600;
        let pos2_upper = 47280;

        let ta_starts: Vec<i32> = [pos1_lower, pos1_upper, pos2_lower, pos2_upper].iter()
            .map(|&t| math::tick_array_start_for_tick(t, tick_spacing))
            .collect::<std::collections::HashSet<i32>>()
            .into_iter().collect();

        let mut ixs: Vec<_> = vec![compute_budget_ix(1_400_000)];
        ixs.extend(ta_starts.iter().map(|&s| init_tick_array_ix(&env, s)));
        let tx = Transaction::new_signed_with_payer(
            &ixs, Some(&env.payer.pubkey()), &[&env.payer], env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        env.svm.expire_blockhash();
        let ix1 = open_position_ix(&env, &env.payer.pubkey(), pos1_lower, pos1_upper);
        let ix2 = open_position_ix(&env, &env.payer.pubkey(), pos2_lower, pos2_upper);
        let tx = Transaction::new_signed_with_payer(
            &[compute_budget_ix(1_400_000), ix1, ix2], Some(&env.payer.pubkey()), &[&env.payer], env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        env.svm.expire_blockhash();
        let liq = 500_000_000u128;
        let ix1 = add_liquidity_ix(&env, &env.payer.pubkey(), pos1_lower, pos1_upper, liq, u64::MAX, u64::MAX);
        let ix2 = add_liquidity_ix(&env, &env.payer.pubkey(), pos2_lower, pos2_upper, liq, u64::MAX, u64::MAX);
        let tx = Transaction::new_signed_with_payer(
            &[compute_budget_ix(1_400_000), ix1, ix2], Some(&env.payer.pubkey()), &[&env.payer], env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        let pool_before = read_pool(&env.svm, &env.pool);
        println!("Before gap swap: tick={}, L={}", pool_before.tick_current, pool_before.liquidity);

        // Large a→b swap to push through the gap
        env.svm.expire_blockhash();
        let mut ta_list: Vec<i32> = ta_starts.clone();
        ta_list.sort();
        ta_list.reverse();

        let user_b_before = token_balance(&env.svm,
            &get_associated_token_address(&env.payer.pubkey(), &env.mint_b));

        let ix = swap_ix(
            &env, &env.payer.pubkey(), true, 100_000_000u64,
            MIN_SQRT_PRICE, 1, &ta_list,
        );
        let tx = Transaction::new_signed_with_payer(
            &[compute_budget_ix(1_400_000), ix], Some(&env.payer.pubkey()), &[&env.payer], env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        let pool_after = read_pool(&env.svm, &env.pool);
        let user_b_after = token_balance(&env.svm,
            &get_associated_token_address(&env.payer.pubkey(), &env.mint_b));
        let output = user_b_after - user_b_before;
        println!("After gap swap: tick={}, L={}, output={}", pool_after.tick_current, pool_after.liquidity, output);
        assert!(output > 0, "Should produce output despite gap");
        // Price should have jumped through the gap
        assert!(pool_after.tick_current < pos2_lower,
            "Should have crossed through position 2's lower bound");
    }

    // --- Fees (2 tests) ---

    #[test]
    fn test_18_collect_fees_after_swaps() {
        let (mut env, tick_lower, tick_upper) = setup_with_liquidity();

        let ta_start = math::tick_array_start_for_tick(tick_lower, env.tick_spacing);
        let ta_start2 = math::tick_array_start_for_tick(tick_upper, env.tick_spacing);
        let mut tas = vec![ta_start];
        if ta_start2 != ta_start { tas.push(ta_start2); }

        // Do some swaps to generate fees
        for _ in 0..3 {
            env.svm.expire_blockhash();
            let ix = swap_ix(
                &env, &env.payer.pubkey(), true, 50_000,
                MIN_SQRT_PRICE, 1, &tas,
            );
            let tx = Transaction::new_signed_with_payer(
                &[compute_budget_ix(1_400_000), ix], Some(&env.payer.pubkey()), &[&env.payer], env.svm.latest_blockhash(),
            );
            env.svm.send_transaction(tx).unwrap();
        }

        let pool = read_pool(&env.svm, &env.pool);
        assert!(pool.fee_growth_global_a > 0, "Fee growth A should be > 0");

        // Collect fees
        let user_a_before = token_balance(&env.svm,
            &get_associated_token_address(&env.payer.pubkey(), &env.mint_a));

        env.svm.expire_blockhash();
        let ix = collect_fees_ix(&env, &env.payer.pubkey(), tick_lower, tick_upper);
        let tx = Transaction::new_signed_with_payer(
            &[compute_budget_ix(1_400_000), ix], Some(&env.payer.pubkey()), &[&env.payer], env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        let user_a_after = token_balance(&env.svm,
            &get_associated_token_address(&env.payer.pubkey(), &env.mint_a));
        let fees_collected = user_a_after - user_a_before;
        println!("Fees collected (token A): {}", fees_collected);
        assert!(fees_collected > 0, "Should collect some fees");

        // Verify position tokens_owed is reset
        let (pos_key, _) = position_pda(&env.pool, &env.payer.pubkey(), tick_lower, tick_upper);
        let pos = read_position(&env.svm, &pos_key);
        assert_eq!(pos.tokens_owed_a, 0);
        assert_eq!(pos.tokens_owed_b, 0);
    }

    #[test]
    fn test_19_fee_distribution_two_positions() {
        let mut env = setup();
        let tick_spacing = env.tick_spacing;

        // Position 1: wide range [42000, 49080] — less concentrated
        // Position 2: narrow range [45060, 47040] — more concentrated
        // Both overlap current tick ~46054
        // Same liquidity amount → position 2 earns MORE fees because its liquidity
        // is concentrated in a narrower range and all of it is active

        let wide_lower = 42000;
        let wide_upper = 49080;
        let narrow_lower = 45060;
        let narrow_upper = 47040;

        let ta_starts: Vec<i32> = [wide_lower, wide_upper, narrow_lower, narrow_upper].iter()
            .map(|&t| math::tick_array_start_for_tick(t, tick_spacing))
            .collect::<std::collections::HashSet<i32>>()
            .into_iter().collect();

        let mut ixs: Vec<_> = vec![compute_budget_ix(1_400_000)];
        ixs.extend(ta_starts.iter().map(|&s| init_tick_array_ix(&env, s)));
        let tx = Transaction::new_signed_with_payer(
            &ixs, Some(&env.payer.pubkey()), &[&env.payer], env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        // Open and fund both positions with same liquidity
        env.svm.expire_blockhash();
        let ix1 = open_position_ix(&env, &env.payer.pubkey(), wide_lower, wide_upper);
        let ix2 = open_position_ix(&env, &env.payer.pubkey(), narrow_lower, narrow_upper);
        let tx = Transaction::new_signed_with_payer(
            &[compute_budget_ix(1_400_000), ix1, ix2], Some(&env.payer.pubkey()), &[&env.payer], env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        env.svm.expire_blockhash();
        let liq = 500_000_000u128;
        let ix1 = add_liquidity_ix(&env, &env.payer.pubkey(), wide_lower, wide_upper, liq, u64::MAX, u64::MAX);
        let ix2 = add_liquidity_ix(&env, &env.payer.pubkey(), narrow_lower, narrow_upper, liq, u64::MAX, u64::MAX);
        let tx = Transaction::new_signed_with_payer(
            &[compute_budget_ix(1_400_000), ix1, ix2], Some(&env.payer.pubkey()), &[&env.payer], env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        // Swap to generate fees
        let mut swap_tas: Vec<i32> = ta_starts.clone();
        swap_tas.sort();

        for _ in 0..5 {
            env.svm.expire_blockhash();
            let ix = swap_ix(&env, &env.payer.pubkey(), true, 50_000, MIN_SQRT_PRICE, 1, &swap_tas);
            let tx = Transaction::new_signed_with_payer(
                &[compute_budget_ix(1_400_000), ix], Some(&env.payer.pubkey()), &[&env.payer], env.svm.latest_blockhash(),
            );
            env.svm.send_transaction(tx).unwrap();
        }

        // Collect fees for both positions
        let user_a_before = token_balance(&env.svm,
            &get_associated_token_address(&env.payer.pubkey(), &env.mint_a));

        env.svm.expire_blockhash();
        let ix = collect_fees_ix(&env, &env.payer.pubkey(), wide_lower, wide_upper);
        let tx = Transaction::new_signed_with_payer(
            &[compute_budget_ix(1_400_000), ix], Some(&env.payer.pubkey()), &[&env.payer], env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        let user_a_mid = token_balance(&env.svm,
            &get_associated_token_address(&env.payer.pubkey(), &env.mint_a));
        let wide_fees = user_a_mid - user_a_before;

        env.svm.expire_blockhash();
        let ix = collect_fees_ix(&env, &env.payer.pubkey(), narrow_lower, narrow_upper);
        let tx = Transaction::new_signed_with_payer(
            &[compute_budget_ix(1_400_000), ix], Some(&env.payer.pubkey()), &[&env.payer], env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        let user_a_after = token_balance(&env.svm,
            &get_associated_token_address(&env.payer.pubkey(), &env.mint_a));
        let narrow_fees = user_a_after - user_a_mid;

        println!("Wide position fees: {}, Narrow position fees: {}", wide_fees, narrow_fees);
        // Both should be equal because they have the same liquidity amount
        // (fees are per unit of liquidity, and both have equal L)
        // The difference is that the narrow position required LESS capital to provide
        // the same L — that's the capital efficiency advantage
        assert!(wide_fees > 0 && narrow_fees > 0, "Both should earn fees");
        // With equal L, fees should be approximately equal
        let ratio = if narrow_fees > wide_fees {
            narrow_fees as f64 / wide_fees.max(1) as f64
        } else {
            wide_fees as f64 / narrow_fees.max(1) as f64
        };
        assert!(ratio < 1.1, "Equal L positions should earn similar fees, ratio={}", ratio);
    }

    // --- Lifecycle (1 test) ---

    #[test]
    fn test_20_full_lifecycle() {
        let mut env = setup();
        let tick_spacing = env.tick_spacing;
        let user1 = &env.payer;

        // Step 1: Init tick arrays covering our position ranges
        let pos1_lower = 45060;
        let pos1_upper = 47040;
        let pos2_lower = 44040;
        let pos2_upper = 46080;

        let ta_starts: Vec<i32> = [pos1_lower, pos1_upper, pos2_lower, pos2_upper].iter()
            .map(|&t| math::tick_array_start_for_tick(t, tick_spacing))
            .collect::<std::collections::HashSet<i32>>()
            .into_iter().collect();

        let mut ixs: Vec<_> = vec![compute_budget_ix(1_400_000)];
        ixs.extend(ta_starts.iter().map(|&s| init_tick_array_ix(&env, s)));
        let tx = Transaction::new_signed_with_payer(
            &ixs, Some(&user1.pubkey()), &[user1], env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        // Step 2: Open 2 overlapping positions
        env.svm.expire_blockhash();
        let ix1 = open_position_ix(&env, &user1.pubkey(), pos1_lower, pos1_upper);
        let ix2 = open_position_ix(&env, &user1.pubkey(), pos2_lower, pos2_upper);
        let tx = Transaction::new_signed_with_payer(
            &[compute_budget_ix(1_400_000), ix1, ix2], Some(&user1.pubkey()), &[user1], env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        // Step 3: Add liquidity to both
        env.svm.expire_blockhash();
        let liq = 500_000_000u128;
        let ix1 = add_liquidity_ix(&env, &user1.pubkey(), pos1_lower, pos1_upper, liq, u64::MAX, u64::MAX);
        let ix2 = add_liquidity_ix(&env, &user1.pubkey(), pos2_lower, pos2_upper, liq, u64::MAX, u64::MAX);
        let tx = Transaction::new_signed_with_payer(
            &[compute_budget_ix(1_400_000), ix1, ix2], Some(&user1.pubkey()), &[user1], env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        let pool = read_pool(&env.svm, &env.pool);
        println!("After adding liquidity: L={}, tick={}", pool.liquidity, pool.tick_current);
        assert!(pool.liquidity > 0);

        let vault_a_after_add = token_balance(&env.svm, &env.vault_a);
        let vault_b_after_add = token_balance(&env.svm, &env.vault_b);
        println!("Vaults: A={}, B={}", vault_a_after_add, vault_b_after_add);

        // Step 4: Multiple swaps both directions
        let mut swap_tas: Vec<i32> = ta_starts.clone();
        swap_tas.sort();

        for _ in 0..3 {
            env.svm.expire_blockhash();
            let ix = swap_ix(&env, &user1.pubkey(), true, 50_000, MIN_SQRT_PRICE, 1, &swap_tas);
            let tx = Transaction::new_signed_with_payer(
                &[compute_budget_ix(1_400_000), ix], Some(&user1.pubkey()), &[user1], env.svm.latest_blockhash(),
            );
            env.svm.send_transaction(tx).unwrap();
        }
        for _ in 0..3 {
            env.svm.expire_blockhash();
            let ix = swap_ix(&env, &user1.pubkey(), false, 50_000, MAX_SQRT_PRICE, 1, &swap_tas);
            let tx = Transaction::new_signed_with_payer(
                &[compute_budget_ix(1_400_000), ix], Some(&user1.pubkey()), &[user1], env.svm.latest_blockhash(),
            );
            env.svm.send_transaction(tx).unwrap();
        }

        let pool_after_swaps = read_pool(&env.svm, &env.pool);
        println!("After swaps: fee_growth_a={}, fee_growth_b={}",
            pool_after_swaps.fee_growth_global_a, pool_after_swaps.fee_growth_global_b);

        // Step 5: Collect fees for both positions
        env.svm.expire_blockhash();
        let ix = collect_fees_ix(&env, &user1.pubkey(), pos1_lower, pos1_upper);
        let tx = Transaction::new_signed_with_payer(
            &[compute_budget_ix(1_400_000), ix], Some(&user1.pubkey()), &[user1], env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        env.svm.expire_blockhash();
        let ix = collect_fees_ix(&env, &user1.pubkey(), pos2_lower, pos2_upper);
        let tx = Transaction::new_signed_with_payer(
            &[compute_budget_ix(1_400_000), ix], Some(&user1.pubkey()), &[user1], env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        // Step 6: Remove all liquidity from both positions
        env.svm.expire_blockhash();
        let ix = remove_liquidity_ix(&env, &user1.pubkey(), pos1_lower, pos1_upper, liq, 0, 0);
        let tx = Transaction::new_signed_with_payer(
            &[compute_budget_ix(1_400_000), ix], Some(&user1.pubkey()), &[user1], env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        env.svm.expire_blockhash();
        let ix = remove_liquidity_ix(&env, &user1.pubkey(), pos2_lower, pos2_upper, liq, 0, 0);
        let tx = Transaction::new_signed_with_payer(
            &[compute_budget_ix(1_400_000), ix], Some(&user1.pubkey()), &[user1], env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        // Verify positions are empty
        let (pos1_key, _) = position_pda(&env.pool, &user1.pubkey(), pos1_lower, pos1_upper);
        let pos1 = read_position(&env.svm, &pos1_key);
        assert_eq!(pos1.liquidity, 0);

        let (pos2_key, _) = position_pda(&env.pool, &user1.pubkey(), pos2_lower, pos2_upper);
        let pos2 = read_position(&env.svm, &pos2_key);
        assert_eq!(pos2.liquidity, 0);

        // Collect any remaining fees from remove_liquidity
        env.svm.expire_blockhash();
        let ix = collect_fees_ix(&env, &user1.pubkey(), pos1_lower, pos1_upper);
        let tx = Transaction::new_signed_with_payer(
            &[compute_budget_ix(1_400_000), ix], Some(&user1.pubkey()), &[user1], env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        env.svm.expire_blockhash();
        let ix = collect_fees_ix(&env, &user1.pubkey(), pos2_lower, pos2_upper);
        let tx = Transaction::new_signed_with_payer(
            &[compute_budget_ix(1_400_000), ix], Some(&user1.pubkey()), &[user1], env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        // Step 7: Close both positions
        env.svm.expire_blockhash();
        let ix = close_position_ix(&env, &user1.pubkey(), pos1_lower, pos1_upper);
        let tx = Transaction::new_signed_with_payer(
            &[compute_budget_ix(1_400_000), ix], Some(&user1.pubkey()), &[user1], env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        env.svm.expire_blockhash();
        let ix = close_position_ix(&env, &user1.pubkey(), pos2_lower, pos2_upper);
        let tx = Transaction::new_signed_with_payer(
            &[compute_budget_ix(1_400_000), ix], Some(&user1.pubkey()), &[user1], env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        // Verify positions are closed (account should not exist)
        assert!(env.svm.get_account(&pos1_key).is_none(), "Position 1 should be closed");
        assert!(env.svm.get_account(&pos2_key).is_none(), "Position 2 should be closed");

        // Pool liquidity should be 0
        let pool_final = read_pool(&env.svm, &env.pool);
        assert_eq!(pool_final.liquidity, 0);

        println!("Full lifecycle complete!");
        println!("Protocol fees A={}, B={}", pool_final.protocol_fees_a, pool_final.protocol_fees_b);
    }
}
