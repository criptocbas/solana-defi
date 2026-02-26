#[cfg(test)]
mod tests {
    use anchor_lang::{system_program, InstructionData, ToAccountMetas};
    use anchor_spl::associated_token::get_associated_token_address;
    use litesvm::LiteSVM;
    use solana_sdk::{
        instruction::Instruction,
        program_pack::Pack,
        pubkey::Pubkey,
        signature::{Keypair, Signer},
        transaction::Transaction,
    };
    use std::str::FromStr;

    use cpamm::constants::*;

    const PROGRAM_ID: &str = "8EpEqMJTjJwFPWbbaSsJi4bDM8z5eZp3aULqdaWppyr9";

    fn program_id() -> Pubkey {
        Pubkey::from_str(PROGRAM_ID).unwrap()
    }

    fn pool_pda(mint_a: &Pubkey, mint_b: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[POOL_SEED, mint_a.as_ref(), mint_b.as_ref()],
            &program_id(),
        )
    }

    fn pool_authority_pda(pool: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[POOL_AUTHORITY_SEED, pool.as_ref()],
            &program_id(),
        )
    }

    fn lp_mint_pda(pool: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[LP_MINT_SEED, pool.as_ref()],
            &program_id(),
        )
    }

    struct TestEnv {
        svm: LiteSVM,
        payer: Keypair,
        mint_a: Pubkey,
        mint_b: Pubkey,
        pool: Pubkey,
        pool_authority: Pubkey,
        lp_mint: Pubkey,
        vault_a: Pubkey,
        vault_b: Pubkey,
        locked_lp_vault: Pubkey,
    }

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
            &spl_token::id(),
            &mint.pubkey(),
            authority,
            None,
            decimals,
        )
        .unwrap();

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
            &payer.pubkey(),
            owner,
            mint,
            &spl_token::id(),
        );
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&payer.pubkey()),
            &[payer],
            svm.latest_blockhash(),
        );
        svm.send_transaction(tx).unwrap();
        ata
    }

    fn mint_tokens(
        svm: &mut LiteSVM,
        payer: &Keypair,
        mint: &Pubkey,
        dest: &Pubkey,
        authority: &Keypair,
        amount: u64,
    ) {
        let ix = spl_token::instruction::mint_to(
            &spl_token::id(),
            mint,
            dest,
            &authority.pubkey(),
            &[],
            amount,
        )
        .unwrap();
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&payer.pubkey()),
            &[payer, authority],
            svm.latest_blockhash(),
        );
        svm.send_transaction(tx).unwrap();
    }

    fn token_balance(svm: &LiteSVM, account: &Pubkey) -> u64 {
        let data = svm.get_account(account).unwrap();
        let token_account = spl_token::state::Account::unpack(&data.data).unwrap();
        token_account.amount
    }

    fn mint_supply(svm: &LiteSVM, mint: &Pubkey) -> u64 {
        let data = svm.get_account(mint).unwrap();
        let mint_state = spl_token::state::Mint::unpack(&data.data).unwrap();
        mint_state.supply
    }

    fn setup() -> TestEnv {
        let mut svm = LiteSVM::new();
        let payer = Keypair::new();
        svm.airdrop(&payer.pubkey(), 100_000_000_000).unwrap();

        // Load the program
        let program_bytes = include_bytes!("../../target/deploy/cpamm.so");
        svm.add_program(program_id(), program_bytes).unwrap();

        // Create mint authority
        let mint_authority = Keypair::new();
        svm.airdrop(&mint_authority.pubkey(), 1_000_000_000).unwrap();

        // Create two token mints
        let m1 = create_mint(&mut svm, &payer, &mint_authority.pubkey(), 6);
        let m2 = create_mint(&mut svm, &payer, &mint_authority.pubkey(), 6);

        // Ensure canonical ordering
        let (mint_a, mint_b) = if m1 < m2 { (m1, m2) } else { (m2, m1) };

        // Derive PDAs
        let (pool, _) = pool_pda(&mint_a, &mint_b);
        let (pool_authority, _) = pool_authority_pda(&pool);
        let (lp_mint, _) = lp_mint_pda(&pool);
        let vault_a = get_associated_token_address(&pool_authority, &mint_a);
        let vault_b = get_associated_token_address(&pool_authority, &mint_b);
        let locked_lp_vault = get_associated_token_address(&pool_authority, &lp_mint);

        // Initialize the pool
        let ix = Instruction {
            program_id: program_id(),
            accounts: cpamm::accounts::InitializePool {
                payer: payer.pubkey(),
                mint_a,
                mint_b,
                pool,
                pool_authority,
                vault_a,
                vault_b,
                lp_mint,
                locked_lp_vault,
                system_program: system_program::ID,
                token_program: spl_token::id(),
                associated_token_program: spl_associated_token_account::id(),
            }
            .to_account_metas(None),
            data: cpamm::instruction::InitializePool {}.data(),
        };

        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&payer.pubkey()),
            &[&payer],
            svm.latest_blockhash(),
        );
        svm.send_transaction(tx).unwrap();

        // Create user token accounts and mint initial tokens
        let user_ata_a = create_ata(&mut svm, &payer, &payer.pubkey(), &mint_a);
        let user_ata_b = create_ata(&mut svm, &payer, &payer.pubkey(), &mint_b);

        mint_tokens(&mut svm, &payer, &mint_a, &user_ata_a, &mint_authority, 10_000_000_000);
        mint_tokens(&mut svm, &payer, &mint_b, &user_ata_b, &mint_authority, 10_000_000_000);

        TestEnv {
            svm,
            payer,
            mint_a,
            mint_b,
            pool,
            pool_authority,
            lp_mint,
            vault_a,
            vault_b,
            locked_lp_vault,
        }
    }

    fn add_liquidity_ix(
        env: &TestEnv,
        user: &Pubkey,
        amount_a: u64,
        amount_b: u64,
        min_lp: u64,
    ) -> Instruction {
        let user_lp_token = get_associated_token_address(user, &env.lp_mint);
        let user_token_a = get_associated_token_address(user, &env.mint_a);
        let user_token_b = get_associated_token_address(user, &env.mint_b);

        Instruction {
            program_id: program_id(),
            accounts: cpamm::accounts::AddLiquidity {
                user: *user,
                pool: env.pool,
                pool_authority: env.pool_authority,
                lp_mint: env.lp_mint,
                vault_a: env.vault_a,
                vault_b: env.vault_b,
                user_token_a,
                user_token_b,
                user_lp_token,
                locked_lp_vault: env.locked_lp_vault,
                system_program: system_program::ID,
                token_program: spl_token::id(),
                associated_token_program: spl_associated_token_account::id(),
            }
            .to_account_metas(None),
            data: cpamm::instruction::AddLiquidity {
                amount_a_desired: amount_a,
                amount_b_desired: amount_b,
                minimum_lp_tokens: min_lp,
            }
            .data(),
        }
    }

    fn swap_ix(
        env: &TestEnv,
        user: &Pubkey,
        input_mint: &Pubkey,
        amount_in: u64,
        min_out: u64,
    ) -> Instruction {
        let (user_token_in, user_token_out) = if *input_mint == env.mint_a {
            (
                get_associated_token_address(user, &env.mint_a),
                get_associated_token_address(user, &env.mint_b),
            )
        } else {
            (
                get_associated_token_address(user, &env.mint_b),
                get_associated_token_address(user, &env.mint_a),
            )
        };

        Instruction {
            program_id: program_id(),
            accounts: cpamm::accounts::Swap {
                user: *user,
                pool: env.pool,
                pool_authority: env.pool_authority,
                vault_a: env.vault_a,
                vault_b: env.vault_b,
                user_token_in,
                user_token_out,
                input_mint: *input_mint,
                token_program: spl_token::id(),
            }
            .to_account_metas(None),
            data: cpamm::instruction::Swap {
                amount_in,
                minimum_amount_out: min_out,
            }
            .data(),
        }
    }

    fn remove_liquidity_ix(
        env: &TestEnv,
        user: &Pubkey,
        lp_burn: u64,
        min_a: u64,
        min_b: u64,
    ) -> Instruction {
        let user_lp_token = get_associated_token_address(user, &env.lp_mint);
        let user_token_a = get_associated_token_address(user, &env.mint_a);
        let user_token_b = get_associated_token_address(user, &env.mint_b);

        Instruction {
            program_id: program_id(),
            accounts: cpamm::accounts::RemoveLiquidity {
                user: *user,
                pool: env.pool,
                pool_authority: env.pool_authority,
                lp_mint: env.lp_mint,
                vault_a: env.vault_a,
                vault_b: env.vault_b,
                user_lp_token,
                user_token_a,
                user_token_b,
                token_program: spl_token::id(),
            }
            .to_account_metas(None),
            data: cpamm::instruction::RemoveLiquidity {
                lp_burn,
                min_amount_a: min_a,
                min_amount_b: min_b,
            }
            .data(),
        }
    }

    // ========== TESTS ==========

    #[test]
    fn test_01_pool_initialization() {
        let env = setup();
        assert_eq!(token_balance(&env.svm, &env.vault_a), 0);
        assert_eq!(token_balance(&env.svm, &env.vault_b), 0);
        assert_eq!(mint_supply(&env.svm, &env.lp_mint), 0);
    }

    #[test]
    fn test_02_first_deposit() {
        let mut env = setup();

        let deposit_a: u64 = 1_000_000;
        let deposit_b: u64 = 4_000_000;
        // sqrt(1_000_000 * 4_000_000) = 2_000_000. LP to user = 2_000_000 - 1_000 = 1_999_000
        let expected_lp = 1_999_000u64;

        let ix = add_liquidity_ix(&env, &env.payer.pubkey(), deposit_a, deposit_b, 0);
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&env.payer.pubkey()),
            &[&env.payer],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        let user_lp = get_associated_token_address(&env.payer.pubkey(), &env.lp_mint);
        assert_eq!(token_balance(&env.svm, &user_lp), expected_lp);
        assert_eq!(token_balance(&env.svm, &env.locked_lp_vault), MINIMUM_LIQUIDITY);
        assert_eq!(token_balance(&env.svm, &env.vault_a), deposit_a);
        assert_eq!(token_balance(&env.svm, &env.vault_b), deposit_b);
        assert_eq!(mint_supply(&env.svm, &env.lp_mint), expected_lp + MINIMUM_LIQUIDITY);
    }

    #[test]
    fn test_03_subsequent_proportional_deposit() {
        let mut env = setup();

        // First deposit: 1:4 ratio
        let ix1 = add_liquidity_ix(&env, &env.payer.pubkey(), 1_000_000, 4_000_000, 0);
        let tx1 = Transaction::new_signed_with_payer(
            &[ix1],
            Some(&env.payer.pubkey()),
            &[&env.payer],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx1).unwrap();

        let supply_before = mint_supply(&env.svm, &env.lp_mint);

        // Second deposit: same 1:4 ratio, double amount
        let ix2 = add_liquidity_ix(&env, &env.payer.pubkey(), 2_000_000, 8_000_000, 0);
        let tx2 = Transaction::new_signed_with_payer(
            &[ix2],
            Some(&env.payer.pubkey()),
            &[&env.payer],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx2).unwrap();

        let supply_after = mint_supply(&env.svm, &env.lp_mint);
        let lp_minted = supply_after - supply_before;
        assert_eq!(lp_minted, supply_before * 2);

        assert_eq!(token_balance(&env.svm, &env.vault_a), 3_000_000);
        assert_eq!(token_balance(&env.svm, &env.vault_b), 12_000_000);
    }

    #[test]
    fn test_04_swap_a_to_b() {
        let mut env = setup();

        let ix = add_liquidity_ix(&env, &env.payer.pubkey(), 1_000_000, 1_000_000, 0);
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&env.payer.pubkey()),
            &[&env.payer],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        let user_b_before = token_balance(
            &env.svm,
            &get_associated_token_address(&env.payer.pubkey(), &env.mint_b),
        );

        let swap_amount: u64 = 10_000;
        let ix = swap_ix(&env, &env.payer.pubkey(), &env.mint_a, swap_amount, 1);
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&env.payer.pubkey()),
            &[&env.payer],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        let user_b_after = token_balance(
            &env.svm,
            &get_associated_token_address(&env.payer.pubkey(), &env.mint_b),
        );
        let received = user_b_after - user_b_before;

        assert!(received > 0);
        assert!(received < swap_amount);

        // Verify k increased (fee accrual)
        let reserve_a = token_balance(&env.svm, &env.vault_a);
        let reserve_b = token_balance(&env.svm, &env.vault_b);
        let k_after = (reserve_a as u128) * (reserve_b as u128);
        let k_before = 1_000_000u128 * 1_000_000u128;
        assert!(k_after > k_before, "k should increase from fees");
    }

    #[test]
    fn test_05_swap_b_to_a() {
        let mut env = setup();

        let ix = add_liquidity_ix(&env, &env.payer.pubkey(), 1_000_000, 1_000_000, 0);
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&env.payer.pubkey()),
            &[&env.payer],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        let user_a_before = token_balance(
            &env.svm,
            &get_associated_token_address(&env.payer.pubkey(), &env.mint_a),
        );

        let ix = swap_ix(&env, &env.payer.pubkey(), &env.mint_b, 10_000, 1);
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&env.payer.pubkey()),
            &[&env.payer],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        let user_a_after = token_balance(
            &env.svm,
            &get_associated_token_address(&env.payer.pubkey(), &env.mint_a),
        );
        let received = user_a_after - user_a_before;
        assert!(received > 0);
        assert!(received < 10_000);
    }

    #[test]
    fn test_06_swap_slippage_exceeded() {
        let mut env = setup();

        let ix = add_liquidity_ix(&env, &env.payer.pubkey(), 1_000_000, 1_000_000, 0);
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&env.payer.pubkey()),
            &[&env.payer],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        // Swap with unreasonably high minimum output
        let ix = swap_ix(&env, &env.payer.pubkey(), &env.mint_a, 10_000, 10_000);
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&env.payer.pubkey()),
            &[&env.payer],
            env.svm.latest_blockhash(),
        );
        let result = env.svm.send_transaction(tx);
        assert!(result.is_err(), "Should fail with slippage exceeded");
    }

    #[test]
    fn test_07_remove_liquidity() {
        let mut env = setup();

        let ix = add_liquidity_ix(&env, &env.payer.pubkey(), 1_000_000, 1_000_000, 0);
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&env.payer.pubkey()),
            &[&env.payer],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        let user_lp = get_associated_token_address(&env.payer.pubkey(), &env.lp_mint);
        let lp_balance = token_balance(&env.svm, &user_lp);

        let user_a_before = token_balance(
            &env.svm,
            &get_associated_token_address(&env.payer.pubkey(), &env.mint_a),
        );
        let user_b_before = token_balance(
            &env.svm,
            &get_associated_token_address(&env.payer.pubkey(), &env.mint_b),
        );

        let burn_amount = lp_balance / 2;
        let ix = remove_liquidity_ix(&env, &env.payer.pubkey(), burn_amount, 0, 0);
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&env.payer.pubkey()),
            &[&env.payer],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        let user_a_after = token_balance(
            &env.svm,
            &get_associated_token_address(&env.payer.pubkey(), &env.mint_a),
        );
        let user_b_after = token_balance(
            &env.svm,
            &get_associated_token_address(&env.payer.pubkey(), &env.mint_b),
        );

        assert!(user_a_after > user_a_before);
        assert!(user_b_after > user_b_before);

        let lp_after = token_balance(&env.svm, &user_lp);
        assert_eq!(lp_after, lp_balance - burn_amount);
    }

    #[test]
    fn test_08_remove_liquidity_slippage() {
        let mut env = setup();

        let ix = add_liquidity_ix(&env, &env.payer.pubkey(), 1_000_000, 1_000_000, 0);
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&env.payer.pubkey()),
            &[&env.payer],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        let user_lp = get_associated_token_address(&env.payer.pubkey(), &env.lp_mint);
        let lp_balance = token_balance(&env.svm, &user_lp);

        let ix = remove_liquidity_ix(
            &env,
            &env.payer.pubkey(),
            lp_balance / 2,
            999_999,
            999_999,
        );
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&env.payer.pubkey()),
            &[&env.payer],
            env.svm.latest_blockhash(),
        );
        let result = env.svm.send_transaction(tx);
        assert!(result.is_err(), "Should fail with slippage");
    }

    #[test]
    fn test_09_large_swap_price_impact() {
        let mut env = setup();

        let ix = add_liquidity_ix(&env, &env.payer.pubkey(), 1_000_000, 1_000_000, 0);
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&env.payer.pubkey()),
            &[&env.payer],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        let user_b_before = token_balance(
            &env.svm,
            &get_associated_token_address(&env.payer.pubkey(), &env.mint_b),
        );

        // Large swap: 500_000 (50% of reserve)
        let ix = swap_ix(&env, &env.payer.pubkey(), &env.mint_a, 500_000, 1);
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&env.payer.pubkey()),
            &[&env.payer],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        let user_b_after = token_balance(
            &env.svm,
            &get_associated_token_address(&env.payer.pubkey(), &env.mint_b),
        );
        let received = user_b_after - user_b_before;

        // With 50% of pool as input, significant price impact
        assert!(received < 340_000, "Large price impact expected, got {}", received);
        assert!(received > 320_000, "Output should be reasonable, got {}", received);
    }

    #[test]
    fn test_10_zero_amount_swap_fails() {
        let mut env = setup();

        let ix = add_liquidity_ix(&env, &env.payer.pubkey(), 1_000_000, 1_000_000, 0);
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&env.payer.pubkey()),
            &[&env.payer],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        let ix = swap_ix(&env, &env.payer.pubkey(), &env.mint_a, 0, 0);
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&env.payer.pubkey()),
            &[&env.payer],
            env.svm.latest_blockhash(),
        );
        let result = env.svm.send_transaction(tx);
        assert!(result.is_err(), "Zero swap should fail");
    }

    #[test]
    fn test_11_invalid_token_order_on_init() {
        let mut svm = LiteSVM::new();
        let payer = Keypair::new();
        svm.airdrop(&payer.pubkey(), 100_000_000_000).unwrap();

        let program_bytes = include_bytes!("../../target/deploy/cpamm.so");
        svm.add_program(program_id(), program_bytes).unwrap();

        let mint_authority = Keypair::new();
        svm.airdrop(&mint_authority.pubkey(), 1_000_000_000).unwrap();

        let m1 = create_mint(&mut svm, &payer, &mint_authority.pubkey(), 6);
        let m2 = create_mint(&mut svm, &payer, &mint_authority.pubkey(), 6);

        // Use reverse order (larger pubkey first)
        let (mint_a, mint_b) = if m1 > m2 { (m1, m2) } else { (m2, m1) };

        let (pool, _) = pool_pda(&mint_a, &mint_b);
        let (pool_authority, _) = pool_authority_pda(&pool);
        let (lp_mint, _) = lp_mint_pda(&pool);
        let vault_a = get_associated_token_address(&pool_authority, &mint_a);
        let vault_b = get_associated_token_address(&pool_authority, &mint_b);
        let locked_lp_vault = get_associated_token_address(&pool_authority, &lp_mint);

        let ix = Instruction {
            program_id: program_id(),
            accounts: cpamm::accounts::InitializePool {
                payer: payer.pubkey(),
                mint_a,
                mint_b,
                pool,
                pool_authority,
                vault_a,
                vault_b,
                lp_mint,
                locked_lp_vault,
                system_program: system_program::ID,
                token_program: spl_token::id(),
                associated_token_program: spl_associated_token_account::id(),
            }
            .to_account_metas(None),
            data: cpamm::instruction::InitializePool {}.data(),
        };

        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&payer.pubkey()),
            &[&payer],
            svm.latest_blockhash(),
        );
        let result = svm.send_transaction(tx);
        assert!(result.is_err(), "Invalid token order should fail");
    }

    #[test]
    fn test_12_fee_accrual_increases_k() {
        let mut env = setup();

        let ix = add_liquidity_ix(&env, &env.payer.pubkey(), 1_000_000, 1_000_000, 0);
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&env.payer.pubkey()),
            &[&env.payer],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        let k_initial =
            token_balance(&env.svm, &env.vault_a) as u128
                * token_balance(&env.svm, &env.vault_b) as u128;

        for _ in 0..5 {
            env.svm.expire_blockhash();
            let ix = swap_ix(&env, &env.payer.pubkey(), &env.mint_a, 50_000, 1);
            let tx = Transaction::new_signed_with_payer(
                &[ix],
                Some(&env.payer.pubkey()),
                &[&env.payer],
                env.svm.latest_blockhash(),
            );
            env.svm.send_transaction(tx).unwrap();

            env.svm.expire_blockhash();
            let ix = swap_ix(&env, &env.payer.pubkey(), &env.mint_b, 50_000, 1);
            let tx = Transaction::new_signed_with_payer(
                &[ix],
                Some(&env.payer.pubkey()),
                &[&env.payer],
                env.svm.latest_blockhash(),
            );
            env.svm.send_transaction(tx).unwrap();
        }

        let k_after =
            token_balance(&env.svm, &env.vault_a) as u128
                * token_balance(&env.svm, &env.vault_b) as u128;

        assert!(k_after > k_initial, "k should grow: {} -> {}", k_initial, k_after);
    }

    #[test]
    fn test_13_full_lifecycle() {
        let mut env = setup();

        // Phase 1: First user deposits
        let deposit_a = 2_000_000u64;
        let deposit_b = 2_000_000u64;

        let ix = add_liquidity_ix(&env, &env.payer.pubkey(), deposit_a, deposit_b, 0);
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&env.payer.pubkey()),
            &[&env.payer],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        let user1_lp = get_associated_token_address(&env.payer.pubkey(), &env.lp_mint);
        assert_eq!(token_balance(&env.svm, &user1_lp), 1_999_000);

        // Phase 2: Swaps generate fees
        for _ in 0..3 {
            env.svm.expire_blockhash();
            let ix = swap_ix(&env, &env.payer.pubkey(), &env.mint_a, 100_000, 1);
            let tx = Transaction::new_signed_with_payer(
                &[ix],
                Some(&env.payer.pubkey()),
                &[&env.payer],
                env.svm.latest_blockhash(),
            );
            env.svm.send_transaction(tx).unwrap();
        }

        let k_after_swaps = token_balance(&env.svm, &env.vault_a) as u128
            * token_balance(&env.svm, &env.vault_b) as u128;
        let k_initial = deposit_a as u128 * deposit_b as u128;
        assert!(k_after_swaps > k_initial, "Fees should increase k");

        // Phase 3: Second user deposits
        env.svm.expire_blockhash();
        let user2 = Keypair::new();
        env.svm.airdrop(&user2.pubkey(), 5_000_000_000).unwrap();

        let user2_ata_a = create_ata(&mut env.svm, &env.payer, &user2.pubkey(), &env.mint_a);
        let user2_ata_b = create_ata(&mut env.svm, &env.payer, &user2.pubkey(), &env.mint_b);

        let transfer_a_ix = spl_token::instruction::transfer(
            &spl_token::id(),
            &get_associated_token_address(&env.payer.pubkey(), &env.mint_a),
            &user2_ata_a,
            &env.payer.pubkey(),
            &[],
            1_000_000,
        )
        .unwrap();
        let transfer_b_ix = spl_token::instruction::transfer(
            &spl_token::id(),
            &get_associated_token_address(&env.payer.pubkey(), &env.mint_b),
            &user2_ata_b,
            &env.payer.pubkey(),
            &[],
            1_000_000,
        )
        .unwrap();
        let tx = Transaction::new_signed_with_payer(
            &[transfer_a_ix, transfer_b_ix],
            Some(&env.payer.pubkey()),
            &[&env.payer],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        env.svm.expire_blockhash();
        let ix = add_liquidity_ix(&env, &user2.pubkey(), 500_000, 500_000, 0);
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&user2.pubkey()),
            &[&user2],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        let user2_lp = get_associated_token_address(&user2.pubkey(), &env.lp_mint);
        let user2_lp_balance = token_balance(&env.svm, &user2_lp);
        assert!(user2_lp_balance > 0, "User2 should receive LP tokens");

        // Phase 4: User1 removes liquidity
        env.svm.expire_blockhash();
        let user1_a_before = token_balance(
            &env.svm,
            &get_associated_token_address(&env.payer.pubkey(), &env.mint_a),
        );
        let user1_b_before = token_balance(
            &env.svm,
            &get_associated_token_address(&env.payer.pubkey(), &env.mint_b),
        );

        let user1_lp_balance = token_balance(&env.svm, &user1_lp);
        let ix = remove_liquidity_ix(&env, &env.payer.pubkey(), user1_lp_balance, 0, 0);
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&env.payer.pubkey()),
            &[&env.payer],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        let user1_a_after = token_balance(
            &env.svm,
            &get_associated_token_address(&env.payer.pubkey(), &env.mint_a),
        );
        let user1_b_after = token_balance(
            &env.svm,
            &get_associated_token_address(&env.payer.pubkey(), &env.mint_b),
        );

        let received_a = user1_a_after - user1_a_before;
        let received_b = user1_b_after - user1_b_before;
        assert!(received_a > 0 && received_b > 0);

        // Phase 5: User2 removes liquidity
        env.svm.expire_blockhash();
        let ix = remove_liquidity_ix(&env, &user2.pubkey(), user2_lp_balance, 0, 0);
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&user2.pubkey()),
            &[&user2],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        let final_lp_supply = mint_supply(&env.svm, &env.lp_mint);
        assert_eq!(final_lp_supply, MINIMUM_LIQUIDITY, "Only locked LP should remain");
    }
}
