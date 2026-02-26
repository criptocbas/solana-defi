#[cfg(test)]
mod tests {
    use anchor_lang::{system_program, InstructionData, ToAccountMetas};
    use anchor_spl::associated_token::get_associated_token_address;
    use litesvm::LiteSVM;
    use solana_sdk::{
        clock::Clock,
        instruction::Instruction,
        program_pack::Pack,
        pubkey::Pubkey,
        signature::{Keypair, Signer},
        transaction::Transaction,
    };
    use std::str::FromStr;

    use klend::constants::*;
    use klend::state::ReserveConfig;

    const PROGRAM_ID: &str = "D91U4ZA4bcSWRNhqAf9oMPBMNYhEkwZNooXPUUZSM68v";

    fn program_id() -> Pubkey {
        Pubkey::from_str(PROGRAM_ID).unwrap()
    }

    // ── PDA helpers ──────────────────────────────────────────────

    fn lending_market_pda(admin: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[LENDING_MARKET_SEED, admin.as_ref()], &program_id())
    }

    fn reserve_pda(lending_market: &Pubkey, token_mint: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[RESERVE_SEED, lending_market.as_ref(), token_mint.as_ref()],
            &program_id(),
        )
    }

    fn reserve_authority_pda(reserve: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[RESERVE_AUTHORITY_SEED, reserve.as_ref()],
            &program_id(),
        )
    }

    fn obligation_pda(lending_market: &Pubkey, owner: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[OBLIGATION_SEED, lending_market.as_ref(), owner.as_ref()],
            &program_id(),
        )
    }

    fn mock_oracle_pda(token_mint: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[MOCK_ORACLE_SEED, token_mint.as_ref()],
            &program_id(),
        )
    }

    // ── SPL helpers ──────────────────────────────────────────────

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

    fn warp_clock(svm: &mut LiteSVM, seconds_forward: i64) {
        let mut clock: Clock = svm.get_sysvar();
        clock.unix_timestamp += seconds_forward;
        svm.set_sysvar(&clock);
    }

    // ── Test config ──────────────────────────────────────────────

    // Pre-computed 1e18-scaled values to avoid const overflow
    const RATE_4_PCT: u64 = 40_000_000_000_000_000;      // 0.04 * 1e18
    const RATE_300_PCT: u64 = 3_000_000_000_000_000_000;  // 3.0 * 1e18
    const UTIL_80_PCT: u64 = 800_000_000_000_000_000;     // 0.80 * 1e18

    fn sol_config() -> ReserveConfig {
        ReserveConfig {
            ltv: 8000,                    // 80%
            liquidation_threshold: 8500,  // 85%
            liquidation_bonus: 500,       // 5%
            reserve_factor: 1000,         // 10%
            r_base: 0,
            r_slope1: RATE_4_PCT,
            r_slope2: RATE_300_PCT,
            u_optimal: UTIL_80_PCT,
            supply_cap: 1_000_000_000_000,     // 1M SOL (9 decimals)
            borrow_cap: 1_000_000_000_000,
            oracle_max_staleness: 120,
        }
    }

    fn usdc_config() -> ReserveConfig {
        ReserveConfig {
            ltv: 8000,
            liquidation_threshold: 8500,
            liquidation_bonus: 500,
            reserve_factor: 1000,
            r_base: 0,
            r_slope1: RATE_4_PCT,
            r_slope2: RATE_300_PCT,
            u_optimal: UTIL_80_PCT,
            supply_cap: 1_000_000_000_000,
            borrow_cap: 1_000_000_000_000,
            oracle_max_staleness: 120,
        }
    }

    // SOL: $100, 9 decimals. USDC: $1, 6 decimals.
    const SOL_PRICE: u64 = 100_000_000;  // $100 * 1e6
    const USDC_PRICE: u64 = 1_000_000;   // $1 * 1e6
    const SOL_DECIMALS: u8 = 9;
    const USDC_DECIMALS: u8 = 6;

    // ── TestEnv ──────────────────────────────────────────────────

    struct TestEnv {
        svm: LiteSVM,
        admin: Keypair,
        mint_authority: Keypair,
        lending_market: Pubkey,
        sol_mint: Pubkey,
        usdc_mint: Pubkey,
        sol_reserve: Pubkey,
        usdc_reserve: Pubkey,
        sol_reserve_authority: Pubkey,
        usdc_reserve_authority: Pubkey,
        sol_vault: Pubkey,
        usdc_vault: Pubkey,
        sol_oracle: Pubkey,
        usdc_oracle: Pubkey,
    }

    // ── Instruction builders ─────────────────────────────────────

    fn init_market_ix(admin: &Pubkey) -> Instruction {
        let (lending_market, _) = lending_market_pda(admin);
        Instruction {
            program_id: program_id(),
            accounts: klend::accounts::InitMarket {
                admin: *admin,
                lending_market,
                system_program: system_program::ID,
            }
            .to_account_metas(None),
            data: klend::instruction::InitMarket {}.data(),
        }
    }

    fn init_mock_oracle_ix(payer: &Pubkey, token_mint: &Pubkey, price: u64, decimals: u8) -> Instruction {
        let (oracle, _) = mock_oracle_pda(token_mint);
        Instruction {
            program_id: program_id(),
            accounts: klend::accounts::InitMockOracle {
                payer: *payer,
                token_mint: *token_mint,
                oracle,
                system_program: system_program::ID,
            }
            .to_account_metas(None),
            data: klend::instruction::InitMockOracle { price, decimals }.data(),
        }
    }

    fn update_mock_oracle_ix(payer: &Pubkey, token_mint: &Pubkey, price: u64) -> Instruction {
        let (oracle, _) = mock_oracle_pda(token_mint);
        Instruction {
            program_id: program_id(),
            accounts: klend::accounts::UpdateMockOracle {
                payer: *payer,
                oracle,
            }
            .to_account_metas(None),
            data: klend::instruction::UpdateMockOracle { price }.data(),
        }
    }

    fn init_reserve_ix(
        admin: &Pubkey,
        lending_market: &Pubkey,
        token_mint: &Pubkey,
        config: ReserveConfig,
    ) -> Instruction {
        let (reserve, _) = reserve_pda(lending_market, token_mint);
        let (reserve_auth, _) = reserve_authority_pda(&reserve);
        let (oracle, _) = mock_oracle_pda(token_mint);
        let token_vault = get_associated_token_address(&reserve_auth, token_mint);

        Instruction {
            program_id: program_id(),
            accounts: klend::accounts::InitReserve {
                admin: *admin,
                lending_market: *lending_market,
                token_mint: *token_mint,
                oracle,
                reserve,
                reserve_authority: reserve_auth,
                token_vault,
                system_program: system_program::ID,
                token_program: spl_token::id(),
                associated_token_program: spl_associated_token_account::id(),
            }
            .to_account_metas(None),
            data: klend::instruction::InitReserve { config }.data(),
        }
    }

    fn refresh_reserve_ix(reserve: &Pubkey, oracle: &Pubkey) -> Instruction {
        Instruction {
            program_id: program_id(),
            accounts: klend::accounts::RefreshReserve {
                reserve: *reserve,
                oracle: *oracle,
            }
            .to_account_metas(None),
            data: klend::instruction::RefreshReserve {}.data(),
        }
    }

    fn deposit_ix(
        user: &Pubkey,
        lending_market: &Pubkey,
        reserve: &Pubkey,
        token_mint: &Pubkey,
        vault: &Pubkey,
        amount: u64,
    ) -> Instruction {
        let (obligation, _) = obligation_pda(lending_market, user);
        let user_token_account = get_associated_token_address(user, token_mint);

        Instruction {
            program_id: program_id(),
            accounts: klend::accounts::Deposit {
                user: *user,
                lending_market: *lending_market,
                reserve: *reserve,
                obligation,
                user_token_account,
                token_vault: *vault,
                token_program: spl_token::id(),
                system_program: system_program::ID,
            }
            .to_account_metas(None),
            data: klend::instruction::Deposit { amount }.data(),
        }
    }

    fn withdraw_ix(
        user: &Pubkey,
        lending_market: &Pubkey,
        reserve: &Pubkey,
        reserve_authority: &Pubkey,
        token_mint: &Pubkey,
        vault: &Pubkey,
        oracle: &Pubkey,
        shares: u64,
    ) -> Instruction {
        let (obligation, _) = obligation_pda(lending_market, user);
        let user_token_account = get_associated_token_address(user, token_mint);

        Instruction {
            program_id: program_id(),
            accounts: klend::accounts::Withdraw {
                user: *user,
                lending_market: *lending_market,
                reserve: *reserve,
                reserve_authority: *reserve_authority,
                obligation,
                owner: *user,
                user_token_account,
                token_vault: *vault,
                oracle: *oracle,
                token_program: spl_token::id(),
            }
            .to_account_metas(None),
            data: klend::instruction::Withdraw { shares }.data(),
        }
    }

    fn borrow_ix(
        user: &Pubkey,
        lending_market: &Pubkey,
        borrow_reserve: &Pubkey,
        borrow_reserve_authority: &Pubkey,
        collateral_reserve: &Pubkey,
        borrow_oracle: &Pubkey,
        collateral_oracle: &Pubkey,
        borrow_mint: &Pubkey,
        borrow_vault: &Pubkey,
        amount: u64,
    ) -> Instruction {
        let (obligation, _) = obligation_pda(lending_market, user);
        let user_token_account = get_associated_token_address(user, borrow_mint);

        Instruction {
            program_id: program_id(),
            accounts: klend::accounts::Borrow {
                user: *user,
                lending_market: *lending_market,
                borrow_reserve: *borrow_reserve,
                borrow_reserve_authority: *borrow_reserve_authority,
                collateral_reserve: *collateral_reserve,
                obligation,
                owner: *user,
                borrow_oracle: *borrow_oracle,
                collateral_oracle: *collateral_oracle,
                user_token_account,
                token_vault: *borrow_vault,
                token_program: spl_token::id(),
            }
            .to_account_metas(None),
            data: klend::instruction::Borrow { amount }.data(),
        }
    }

    fn repay_ix(
        user: &Pubkey,
        obligation_owner: &Pubkey,
        lending_market: &Pubkey,
        reserve: &Pubkey,
        token_mint: &Pubkey,
        vault: &Pubkey,
        amount: u64,
    ) -> Instruction {
        let (obligation, _) = obligation_pda(lending_market, obligation_owner);
        let user_token_account = get_associated_token_address(user, token_mint);

        Instruction {
            program_id: program_id(),
            accounts: klend::accounts::Repay {
                user: *user,
                lending_market: *lending_market,
                reserve: *reserve,
                obligation,
                obligation_owner: *obligation_owner,
                user_token_account,
                token_vault: *vault,
                token_program: spl_token::id(),
            }
            .to_account_metas(None),
            data: klend::instruction::Repay { amount }.data(),
        }
    }

    fn liquidate_ix(
        liquidator: &Pubkey,
        obligation_owner: &Pubkey,
        lending_market: &Pubkey,
        debt_reserve: &Pubkey,
        collateral_reserve: &Pubkey,
        collateral_reserve_authority: &Pubkey,
        debt_oracle: &Pubkey,
        collateral_oracle: &Pubkey,
        debt_mint: &Pubkey,
        collateral_mint: &Pubkey,
        debt_vault: &Pubkey,
        collateral_vault: &Pubkey,
        amount: u64,
    ) -> Instruction {
        let (obligation, _) = obligation_pda(lending_market, obligation_owner);
        let liquidator_debt_token = get_associated_token_address(liquidator, debt_mint);
        let liquidator_collateral_token = get_associated_token_address(liquidator, collateral_mint);

        Instruction {
            program_id: program_id(),
            accounts: klend::accounts::Liquidate {
                liquidator: *liquidator,
                lending_market: *lending_market,
                debt_reserve: *debt_reserve,
                collateral_reserve: *collateral_reserve,
                collateral_reserve_authority: *collateral_reserve_authority,
                obligation,
                obligation_owner: *obligation_owner,
                debt_oracle: *debt_oracle,
                collateral_oracle: *collateral_oracle,
                liquidator_debt_token,
                debt_vault: *debt_vault,
                liquidator_collateral_token,
                collateral_vault: *collateral_vault,
                token_program: spl_token::id(),
            }
            .to_account_metas(None),
            data: klend::instruction::Liquidate { amount }.data(),
        }
    }

    // ── Setup ────────────────────────────────────────────────────

    fn setup() -> TestEnv {
        let mut svm = LiteSVM::new();
        let admin = Keypair::new();
        svm.airdrop(&admin.pubkey(), 100_000_000_000).unwrap();

        // Load program
        let program_bytes = include_bytes!("../../target/deploy/klend.so");
        svm.add_program(program_id(), program_bytes).unwrap();

        // Create mint authority
        let mint_authority = Keypair::new();
        svm.airdrop(&mint_authority.pubkey(), 1_000_000_000).unwrap();

        // Create SOL and USDC mints
        let sol_mint = create_mint(&mut svm, &admin, &mint_authority.pubkey(), SOL_DECIMALS);
        let usdc_mint = create_mint(&mut svm, &admin, &mint_authority.pubkey(), USDC_DECIMALS);

        // Init lending market
        let ix = init_market_ix(&admin.pubkey());
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&admin.pubkey()),
            &[&admin],
            svm.latest_blockhash(),
        );
        svm.send_transaction(tx).unwrap();

        let (lending_market, _) = lending_market_pda(&admin.pubkey());

        // Init oracles
        svm.expire_blockhash();
        let ix1 = init_mock_oracle_ix(&admin.pubkey(), &sol_mint, SOL_PRICE, SOL_DECIMALS);
        let ix2 = init_mock_oracle_ix(&admin.pubkey(), &usdc_mint, USDC_PRICE, USDC_DECIMALS);
        let tx = Transaction::new_signed_with_payer(
            &[ix1, ix2],
            Some(&admin.pubkey()),
            &[&admin],
            svm.latest_blockhash(),
        );
        svm.send_transaction(tx).unwrap();

        let (sol_oracle, _) = mock_oracle_pda(&sol_mint);
        let (usdc_oracle, _) = mock_oracle_pda(&usdc_mint);

        // Init reserves
        svm.expire_blockhash();
        let ix = init_reserve_ix(&admin.pubkey(), &lending_market, &sol_mint, sol_config());
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&admin.pubkey()),
            &[&admin],
            svm.latest_blockhash(),
        );
        svm.send_transaction(tx).unwrap();

        svm.expire_blockhash();
        let ix = init_reserve_ix(&admin.pubkey(), &lending_market, &usdc_mint, usdc_config());
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&admin.pubkey()),
            &[&admin],
            svm.latest_blockhash(),
        );
        svm.send_transaction(tx).unwrap();

        let (sol_reserve, _) = reserve_pda(&lending_market, &sol_mint);
        let (usdc_reserve, _) = reserve_pda(&lending_market, &usdc_mint);
        let (sol_reserve_authority, _) = reserve_authority_pda(&sol_reserve);
        let (usdc_reserve_authority, _) = reserve_authority_pda(&usdc_reserve);
        let sol_vault = get_associated_token_address(&sol_reserve_authority, &sol_mint);
        let usdc_vault = get_associated_token_address(&usdc_reserve_authority, &usdc_mint);

        TestEnv {
            svm,
            admin,
            mint_authority,
            lending_market,
            sol_mint,
            usdc_mint,
            sol_reserve,
            usdc_reserve,
            sol_reserve_authority,
            usdc_reserve_authority,
            sol_vault,
            usdc_vault,
            sol_oracle,
            usdc_oracle,
        }
    }

    /// Helper: fund a user with SOL and USDC tokens + ATAs
    fn fund_user(env: &mut TestEnv, user: &Keypair, sol_amount: u64, usdc_amount: u64) {
        env.svm.airdrop(&user.pubkey(), 10_000_000_000).unwrap();

        if sol_amount > 0 {
            let ata = create_ata(&mut env.svm, &env.admin, &user.pubkey(), &env.sol_mint);
            mint_tokens(
                &mut env.svm,
                &env.admin,
                &env.sol_mint,
                &ata,
                &env.mint_authority,
                sol_amount,
            );
        }
        if usdc_amount > 0 {
            env.svm.expire_blockhash();
            let ata = create_ata(&mut env.svm, &env.admin, &user.pubkey(), &env.usdc_mint);
            mint_tokens(
                &mut env.svm,
                &env.admin,
                &env.usdc_mint,
                &ata,
                &env.mint_authority,
                usdc_amount,
            );
        }
    }

    /// Helper: refresh both reserves using env's admin as signer
    fn refresh_both(env: &mut TestEnv) {
        env.svm.expire_blockhash();
        let ix1 = refresh_reserve_ix(&env.sol_reserve, &env.sol_oracle);
        let ix2 = refresh_reserve_ix(&env.usdc_reserve, &env.usdc_oracle);
        let tx = Transaction::new_signed_with_payer(
            &[ix1, ix2],
            Some(&env.admin.pubkey()),
            &[&env.admin],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();
    }

    // ========== TESTS ==========

    #[test]
    fn test_01_init_market() {
        let env = setup();
        let (lending_market, _) = lending_market_pda(&env.admin.pubkey());
        let account = env.svm.get_account(&lending_market).unwrap();
        // Account exists and has data
        assert!(account.data.len() > 8);
        println!("Market initialized at {}", lending_market);
    }

    #[test]
    fn test_02_init_reserves() {
        let env = setup();
        // Verify SOL reserve exists
        let account = env.svm.get_account(&env.sol_reserve).unwrap();
        assert!(account.data.len() > 8);
        // Verify USDC reserve exists
        let account = env.svm.get_account(&env.usdc_reserve).unwrap();
        assert!(account.data.len() > 8);
        // Verify vaults exist
        assert_eq!(token_balance(&env.svm, &env.sol_vault), 0);
        assert_eq!(token_balance(&env.svm, &env.usdc_vault), 0);
        println!("SOL reserve: {}, USDC reserve: {}", env.sol_reserve, env.usdc_reserve);
    }

    #[test]
    fn test_03_deposit_sol_first() {
        let mut env = setup();
        let user = Keypair::new();
        let deposit_amount = 10_000_000_000u64; // 10 SOL (9 decimals)
        fund_user(&mut env, &user, deposit_amount, 0);

        // Refresh reserve before deposit
        refresh_both(&mut env);

        env.svm.expire_blockhash();
        let ix = deposit_ix(
            &user.pubkey(),
            &env.lending_market,
            &env.sol_reserve,
            &env.sol_mint,
            &env.sol_vault,
            deposit_amount,
        );
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&user.pubkey()),
            &[&user],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        // Verify vault received tokens
        assert_eq!(token_balance(&env.svm, &env.sol_vault), deposit_amount);
        println!("Deposited {} SOL tokens", deposit_amount);
    }

    #[test]
    fn test_04_second_deposit_exchange_rate() {
        let mut env = setup();
        let user1 = Keypair::new();
        let user2 = Keypair::new();
        let deposit_amount = 10_000_000_000u64;
        fund_user(&mut env, &user1, deposit_amount, 0);
        fund_user(&mut env, &user2, deposit_amount, 0);

        // User1 deposits
        refresh_both(&mut env);
        env.svm.expire_blockhash();
        let ix = deposit_ix(
            &user1.pubkey(),
            &env.lending_market,
            &env.sol_reserve,
            &env.sol_mint,
            &env.sol_vault,
            deposit_amount,
        );
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&user1.pubkey()),
            &[&user1],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        let vault_before = token_balance(&env.svm, &env.sol_vault);

        // User2 deposits same amount
        refresh_both(&mut env);
        env.svm.expire_blockhash();
        let ix = deposit_ix(
            &user2.pubkey(),
            &env.lending_market,
            &env.sol_reserve,
            &env.sol_mint,
            &env.sol_vault,
            deposit_amount,
        );
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&user2.pubkey()),
            &[&user2],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        // Vault should have 2x
        assert_eq!(
            token_balance(&env.svm, &env.sol_vault),
            vault_before + deposit_amount
        );
        println!("Second deposit at ~1:1 exchange rate");
    }

    #[test]
    fn test_05_withdraw() {
        let mut env = setup();
        let user = Keypair::new();
        let deposit_amount = 10_000_000_000u64;
        fund_user(&mut env, &user, deposit_amount, 0);

        // Deposit
        refresh_both(&mut env);
        env.svm.expire_blockhash();
        let ix = deposit_ix(
            &user.pubkey(),
            &env.lending_market,
            &env.sol_reserve,
            &env.sol_mint,
            &env.sol_vault,
            deposit_amount,
        );
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&user.pubkey()),
            &[&user],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        let user_sol_ata = get_associated_token_address(&user.pubkey(), &env.sol_mint);
        let user_before = token_balance(&env.svm, &user_sol_ata);

        // Read obligation to get shares
        // With virtual shares: shares ≈ deposit_amount (since first deposit, total_shares=0, total_assets=0)
        // shares = amount * (0 + 1) / (0 + 1) = amount
        let withdraw_shares = deposit_amount; // approximate

        // Withdraw all shares
        refresh_both(&mut env);
        env.svm.expire_blockhash();
        let ix = withdraw_ix(
            &user.pubkey(),
            &env.lending_market,
            &env.sol_reserve,
            &env.sol_reserve_authority,
            &env.sol_mint,
            &env.sol_vault,
            &env.sol_oracle,
            withdraw_shares,
        );
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&user.pubkey()),
            &[&user],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        let user_after = token_balance(&env.svm, &user_sol_ata);
        let received = user_after - user_before;
        // Due to virtual shares rounding, we may get back slightly less
        assert!(received > 0);
        assert!(received <= deposit_amount);
        println!("Withdrew {} (deposited {})", received, deposit_amount);
    }

    #[test]
    fn test_06_borrow_usdc_against_sol() {
        let mut env = setup();
        let user = Keypair::new();
        // Deposit 10 SOL ($1000 at $100/SOL), borrow $500 USDC -> HF ~ 1.7
        let sol_deposit = 10_000_000_000u64; // 10 SOL
        let usdc_borrow = 500_000_000u64;    // 500 USDC

        fund_user(&mut env, &user, sol_deposit, 0);
        // Also create user's USDC ATA (needed to receive borrowed USDC)
        env.svm.expire_blockhash();
        create_ata(&mut env.svm, &env.admin, &user.pubkey(), &env.usdc_mint);

        // Need USDC liquidity in the reserve -- deposit as admin
        let admin_usdc = 1_000_000_000_000u64; // 1M USDC
        env.svm.expire_blockhash();
        let admin_usdc_ata = create_ata(&mut env.svm, &env.admin, &env.admin.pubkey(), &env.usdc_mint);
        mint_tokens(&mut env.svm, &env.admin, &env.usdc_mint, &admin_usdc_ata, &env.mint_authority, admin_usdc);

        // Refresh + deposit USDC as admin
        refresh_both(&mut env);
        env.svm.expire_blockhash();
        let ix = deposit_ix(
            &env.admin.pubkey(),
            &env.lending_market,
            &env.usdc_reserve,
            &env.usdc_mint,
            &env.usdc_vault,
            admin_usdc,
        );
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&env.admin.pubkey()),
            &[&env.admin],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        // User deposits SOL
        refresh_both(&mut env);
        env.svm.expire_blockhash();
        let ix = deposit_ix(
            &user.pubkey(),
            &env.lending_market,
            &env.sol_reserve,
            &env.sol_mint,
            &env.sol_vault,
            sol_deposit,
        );
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&user.pubkey()),
            &[&user],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        // User borrows USDC
        refresh_both(&mut env);
        env.svm.expire_blockhash();
        let ix = borrow_ix(
            &user.pubkey(),
            &env.lending_market,
            &env.usdc_reserve,
            &env.usdc_reserve_authority,
            &env.sol_reserve,
            &env.usdc_oracle,
            &env.sol_oracle,
            &env.usdc_mint,
            &env.usdc_vault,
            usdc_borrow,
        );
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&user.pubkey()),
            &[&user],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        let user_usdc = get_associated_token_address(&user.pubkey(), &env.usdc_mint);
        assert_eq!(token_balance(&env.svm, &user_usdc), usdc_borrow);
        println!("Borrowed {} USDC against {} SOL", usdc_borrow, sol_deposit);
    }

    #[test]
    fn test_07_borrow_rejected_insufficient_collateral() {
        let mut env = setup();
        let user = Keypair::new();
        // Deposit 1 SOL ($100), try to borrow $100 USDC. HF would be < 1.0
        let sol_deposit = 1_000_000_000u64; // 1 SOL
        let usdc_borrow = 100_000_000u64;   // 100 USDC (= $100, but with 85% LT -> $85 weighted)

        fund_user(&mut env, &user, sol_deposit, 0);
        env.svm.expire_blockhash();
        create_ata(&mut env.svm, &env.admin, &user.pubkey(), &env.usdc_mint);

        // Seed USDC reserve
        env.svm.expire_blockhash();
        let admin_usdc_ata = create_ata(&mut env.svm, &env.admin, &env.admin.pubkey(), &env.usdc_mint);
        mint_tokens(&mut env.svm, &env.admin, &env.usdc_mint, &admin_usdc_ata, &env.mint_authority, 1_000_000_000_000);

        refresh_both(&mut env);
        env.svm.expire_blockhash();
        let ix = deposit_ix(
            &env.admin.pubkey(),
            &env.lending_market,
            &env.usdc_reserve,
            &env.usdc_mint,
            &env.usdc_vault,
            1_000_000_000_000,
        );
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&env.admin.pubkey()),
            &[&env.admin],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        // Deposit SOL
        refresh_both(&mut env);
        env.svm.expire_blockhash();
        let ix = deposit_ix(
            &user.pubkey(),
            &env.lending_market,
            &env.sol_reserve,
            &env.sol_mint,
            &env.sol_vault,
            sol_deposit,
        );
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&user.pubkey()),
            &[&user],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        // Try to borrow $100 against $100 collateral (85% weighted = $85 < $100)
        refresh_both(&mut env);
        env.svm.expire_blockhash();
        let ix = borrow_ix(
            &user.pubkey(),
            &env.lending_market,
            &env.usdc_reserve,
            &env.usdc_reserve_authority,
            &env.sol_reserve,
            &env.usdc_oracle,
            &env.sol_oracle,
            &env.usdc_mint,
            &env.usdc_vault,
            usdc_borrow,
        );
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&user.pubkey()),
            &[&user],
            env.svm.latest_blockhash(),
        );
        let result = env.svm.send_transaction(tx);
        assert!(result.is_err(), "Should reject borrow: HF < 1.0");
        println!("Borrow correctly rejected: insufficient collateral");
    }

    #[test]
    fn test_08_partial_repay() {
        let mut env = setup();
        let user = Keypair::new();
        let sol_deposit = 10_000_000_000u64;
        let usdc_borrow = 500_000_000u64;

        fund_user(&mut env, &user, sol_deposit, 1_000_000_000); // extra USDC for repay

        // Seed USDC reserve
        env.svm.expire_blockhash();
        let admin_usdc_ata = create_ata(&mut env.svm, &env.admin, &env.admin.pubkey(), &env.usdc_mint);
        mint_tokens(&mut env.svm, &env.admin, &env.usdc_mint, &admin_usdc_ata, &env.mint_authority, 1_000_000_000_000);
        refresh_both(&mut env);
        env.svm.expire_blockhash();
        let ix = deposit_ix(
            &env.admin.pubkey(),
            &env.lending_market,
            &env.usdc_reserve,
            &env.usdc_mint,
            &env.usdc_vault,
            1_000_000_000_000,
        );
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&env.admin.pubkey()),
            &[&env.admin],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        // Deposit SOL + borrow USDC
        refresh_both(&mut env);
        env.svm.expire_blockhash();
        let ix = deposit_ix(
            &user.pubkey(),
            &env.lending_market,
            &env.sol_reserve,
            &env.sol_mint,
            &env.sol_vault,
            sol_deposit,
        );
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&user.pubkey()),
            &[&user],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        refresh_both(&mut env);
        env.svm.expire_blockhash();
        let ix = borrow_ix(
            &user.pubkey(),
            &env.lending_market,
            &env.usdc_reserve,
            &env.usdc_reserve_authority,
            &env.sol_reserve,
            &env.usdc_oracle,
            &env.sol_oracle,
            &env.usdc_mint,
            &env.usdc_vault,
            usdc_borrow,
        );
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&user.pubkey()),
            &[&user],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        let vault_before = token_balance(&env.svm, &env.usdc_vault);

        // Partial repay: 250 USDC
        let repay_amount = 250_000_000u64;
        refresh_both(&mut env);
        env.svm.expire_blockhash();
        let ix = repay_ix(
            &user.pubkey(),
            &user.pubkey(),
            &env.lending_market,
            &env.usdc_reserve,
            &env.usdc_mint,
            &env.usdc_vault,
            repay_amount,
        );
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&user.pubkey()),
            &[&user],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        let vault_after = token_balance(&env.svm, &env.usdc_vault);
        assert_eq!(vault_after - vault_before, repay_amount);
        println!("Partial repay of {} USDC succeeded", repay_amount);
    }

    #[test]
    fn test_09_full_repay() {
        let mut env = setup();
        let user = Keypair::new();
        let sol_deposit = 10_000_000_000u64;
        let usdc_borrow = 500_000_000u64;

        fund_user(&mut env, &user, sol_deposit, 1_000_000_000);

        // Seed USDC reserve
        env.svm.expire_blockhash();
        let admin_usdc_ata = create_ata(&mut env.svm, &env.admin, &env.admin.pubkey(), &env.usdc_mint);
        mint_tokens(&mut env.svm, &env.admin, &env.usdc_mint, &admin_usdc_ata, &env.mint_authority, 1_000_000_000_000);
        refresh_both(&mut env);
        env.svm.expire_blockhash();
        let ix = deposit_ix(
            &env.admin.pubkey(),
            &env.lending_market,
            &env.usdc_reserve,
            &env.usdc_mint,
            &env.usdc_vault,
            1_000_000_000_000,
        );
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&env.admin.pubkey()),
            &[&env.admin],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        // Deposit + borrow
        refresh_both(&mut env);
        env.svm.expire_blockhash();
        let ix = deposit_ix(
            &user.pubkey(),
            &env.lending_market,
            &env.sol_reserve,
            &env.sol_mint,
            &env.sol_vault,
            sol_deposit,
        );
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&user.pubkey()),
            &[&user],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        refresh_both(&mut env);
        env.svm.expire_blockhash();
        let ix = borrow_ix(
            &user.pubkey(),
            &env.lending_market,
            &env.usdc_reserve,
            &env.usdc_reserve_authority,
            &env.sol_reserve,
            &env.usdc_oracle,
            &env.sol_oracle,
            &env.usdc_mint,
            &env.usdc_vault,
            usdc_borrow,
        );
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&user.pubkey()),
            &[&user],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        // Full repay (use u64::MAX to repay everything, gets capped to debt)
        refresh_both(&mut env);
        env.svm.expire_blockhash();
        let ix = repay_ix(
            &user.pubkey(),
            &user.pubkey(),
            &env.lending_market,
            &env.usdc_reserve,
            &env.usdc_mint,
            &env.usdc_vault,
            u64::MAX,
        );
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&user.pubkey()),
            &[&user],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        println!("Full repay succeeded, borrow entry should be removed");
    }

    #[test]
    fn test_10_interest_accrual() {
        let mut env = setup();
        let user = Keypair::new();
        let sol_deposit = 10_000_000_000u64;
        let usdc_borrow = 500_000_000u64;

        fund_user(&mut env, &user, sol_deposit, 1_000_000_000);

        // Seed USDC reserve
        env.svm.expire_blockhash();
        let admin_usdc_ata = create_ata(&mut env.svm, &env.admin, &env.admin.pubkey(), &env.usdc_mint);
        mint_tokens(&mut env.svm, &env.admin, &env.usdc_mint, &admin_usdc_ata, &env.mint_authority, 1_000_000_000_000);
        refresh_both(&mut env);
        env.svm.expire_blockhash();
        let ix = deposit_ix(
            &env.admin.pubkey(),
            &env.lending_market,
            &env.usdc_reserve,
            &env.usdc_mint,
            &env.usdc_vault,
            1_000_000_000_000,
        );
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&env.admin.pubkey()),
            &[&env.admin],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        // Deposit + borrow
        refresh_both(&mut env);
        env.svm.expire_blockhash();
        let ix = deposit_ix(
            &user.pubkey(),
            &env.lending_market,
            &env.sol_reserve,
            &env.sol_mint,
            &env.sol_vault,
            sol_deposit,
        );
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&user.pubkey()),
            &[&user],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        refresh_both(&mut env);
        env.svm.expire_blockhash();
        let ix = borrow_ix(
            &user.pubkey(),
            &env.lending_market,
            &env.usdc_reserve,
            &env.usdc_reserve_authority,
            &env.sol_reserve,
            &env.usdc_oracle,
            &env.sol_oracle,
            &env.usdc_mint,
            &env.usdc_vault,
            usdc_borrow,
        );
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&user.pubkey()),
            &[&user],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        // Warp 1 year and refresh
        warp_clock(&mut env.svm, 365 * 24 * 3600);

        // Need to also update oracle timestamp so it's not stale
        env.svm.expire_blockhash();
        let ix1 = update_mock_oracle_ix(&env.admin.pubkey(), &env.sol_mint, SOL_PRICE);
        let ix2 = update_mock_oracle_ix(&env.admin.pubkey(), &env.usdc_mint, USDC_PRICE);
        let tx = Transaction::new_signed_with_payer(
            &[ix1, ix2],
            Some(&env.admin.pubkey()),
            &[&env.admin],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        // Refresh USDC reserve (where the borrow is)
        env.svm.expire_blockhash();
        let ix = refresh_reserve_ix(&env.usdc_reserve, &env.usdc_oracle);
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&env.admin.pubkey()),
            &[&env.admin],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        // Deserialize reserve to check
        let reserve_data = env.svm.get_account(&env.usdc_reserve).unwrap();
        let reserve: klend::state::Reserve =
            anchor_lang::AccountDeserialize::try_deserialize(&mut &reserve_data.data[..]).unwrap();

        // With 500M borrowed out of 1T total, utilization = 500M/1T = 0.05%
        // Rate model: U < U_optimal (80%), so rate = r_base + (U/U_opt)*slope1
        // = 0 + (0.0005/0.8)*0.04 = very small
        // After 1 year, borrowed should increase
        println!("Borrowed after 1yr: {} (was {})", reserve.borrowed_liquidity, usdc_borrow);
        println!("Protocol fees: {}", reserve.accumulated_protocol_fees);
        println!("Borrow index: {}", reserve.cumulative_borrow_index);
        assert!(
            reserve.borrowed_liquidity > usdc_borrow,
            "Interest should have accrued"
        );
        assert!(
            reserve.accumulated_protocol_fees > 0,
            "Protocol fees should have accrued"
        );
        assert!(
            reserve.cumulative_borrow_index > SCALE as u128,
            "Borrow index should have increased"
        );
    }

    #[test]
    fn test_11_exchange_rate_increases_with_interest() {
        let mut env = setup();
        let depositor = Keypair::new();
        let borrower = Keypair::new();
        let usdc_deposit = 1_000_000_000u64; // 1000 USDC
        let usdc_borrow = 500_000_000u64;    // 500 USDC

        fund_user(&mut env, &depositor, 0, usdc_deposit);
        fund_user(&mut env, &borrower, 10_000_000_000, usdc_deposit); // SOL for collateral + USDC for repay

        // Depositor deposits USDC
        refresh_both(&mut env);
        env.svm.expire_blockhash();
        let ix = deposit_ix(
            &depositor.pubkey(),
            &env.lending_market,
            &env.usdc_reserve,
            &env.usdc_mint,
            &env.usdc_vault,
            usdc_deposit,
        );
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&depositor.pubkey()),
            &[&depositor],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        // Borrower deposits SOL as collateral
        refresh_both(&mut env);
        env.svm.expire_blockhash();
        let ix = deposit_ix(
            &borrower.pubkey(),
            &env.lending_market,
            &env.sol_reserve,
            &env.sol_mint,
            &env.sol_vault,
            10_000_000_000,
        );
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&borrower.pubkey()),
            &[&borrower],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        // Borrower borrows USDC
        refresh_both(&mut env);
        env.svm.expire_blockhash();
        let ix = borrow_ix(
            &borrower.pubkey(),
            &env.lending_market,
            &env.usdc_reserve,
            &env.usdc_reserve_authority,
            &env.sol_reserve,
            &env.usdc_oracle,
            &env.sol_oracle,
            &env.usdc_mint,
            &env.usdc_vault,
            usdc_borrow,
        );
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&borrower.pubkey()),
            &[&borrower],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        // Read reserve before warp
        let reserve_data = env.svm.get_account(&env.usdc_reserve).unwrap();
        let reserve_before: klend::state::Reserve =
            anchor_lang::AccountDeserialize::try_deserialize(&mut &reserve_data.data[..]).unwrap();
        let assets_before = reserve_before.total_assets();

        // Warp 1 year
        warp_clock(&mut env.svm, 365 * 24 * 3600);
        env.svm.expire_blockhash();
        let ix1 = update_mock_oracle_ix(&env.admin.pubkey(), &env.sol_mint, SOL_PRICE);
        let ix2 = update_mock_oracle_ix(&env.admin.pubkey(), &env.usdc_mint, USDC_PRICE);
        let tx = Transaction::new_signed_with_payer(
            &[ix1, ix2],
            Some(&env.admin.pubkey()),
            &[&env.admin],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        env.svm.expire_blockhash();
        let ix = refresh_reserve_ix(&env.usdc_reserve, &env.usdc_oracle);
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&env.admin.pubkey()),
            &[&env.admin],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        // Read reserve after
        let reserve_data = env.svm.get_account(&env.usdc_reserve).unwrap();
        let reserve_after: klend::state::Reserve =
            anchor_lang::AccountDeserialize::try_deserialize(&mut &reserve_data.data[..]).unwrap();
        let assets_after = reserve_after.total_assets();

        // Exchange rate = total_assets / total_shares
        // Since shares didn't change but assets grew (interest), exchange rate increased
        assert!(
            assets_after > assets_before,
            "Total assets should increase: {} -> {}",
            assets_before,
            assets_after
        );
        println!(
            "Exchange rate increased: assets {} -> {} (shares unchanged)",
            assets_before, assets_after
        );
    }

    #[test]
    fn test_12_liquidation_unhealthy() {
        let mut env = setup();
        let user = Keypair::new();
        let liquidator = Keypair::new();

        // User deposits 1 SOL ($100), borrows 80 USDC
        let sol_deposit = 1_000_000_000u64; // 1 SOL
        let usdc_borrow = 80_000_000u64;    // 80 USDC

        fund_user(&mut env, &user, sol_deposit, 0);
        env.svm.expire_blockhash();
        create_ata(&mut env.svm, &env.admin, &user.pubkey(), &env.usdc_mint);

        // Seed USDC reserve + fund liquidator
        env.svm.expire_blockhash();
        let admin_usdc_ata = create_ata(&mut env.svm, &env.admin, &env.admin.pubkey(), &env.usdc_mint);
        mint_tokens(&mut env.svm, &env.admin, &env.usdc_mint, &admin_usdc_ata, &env.mint_authority, 1_000_000_000_000);
        refresh_both(&mut env);
        env.svm.expire_blockhash();
        let ix = deposit_ix(
            &env.admin.pubkey(),
            &env.lending_market,
            &env.usdc_reserve,
            &env.usdc_mint,
            &env.usdc_vault,
            1_000_000_000_000,
        );
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&env.admin.pubkey()),
            &[&env.admin],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        // User deposits SOL and borrows
        refresh_both(&mut env);
        env.svm.expire_blockhash();
        let ix = deposit_ix(
            &user.pubkey(),
            &env.lending_market,
            &env.sol_reserve,
            &env.sol_mint,
            &env.sol_vault,
            sol_deposit,
        );
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&user.pubkey()),
            &[&user],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        refresh_both(&mut env);
        env.svm.expire_blockhash();
        let ix = borrow_ix(
            &user.pubkey(),
            &env.lending_market,
            &env.usdc_reserve,
            &env.usdc_reserve_authority,
            &env.sol_reserve,
            &env.usdc_oracle,
            &env.sol_oracle,
            &env.usdc_mint,
            &env.usdc_vault,
            usdc_borrow,
        );
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&user.pubkey()),
            &[&user],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        // Drop SOL price from $100 to $50 -> HF drops
        // Collateral value: 1 SOL * $50 = $50, weighted: $50 * 85% = $42.50
        // Debt: 80 USDC = $80. HF = 42.5/80 = 0.53 < 1.0
        env.svm.expire_blockhash();
        let ix = update_mock_oracle_ix(&env.admin.pubkey(), &env.sol_mint, 50_000_000); // $50
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&env.admin.pubkey()),
            &[&env.admin],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        // Fund liquidator with USDC
        fund_user(&mut env, &liquidator, 0, 1_000_000_000);
        env.svm.expire_blockhash();
        create_ata(&mut env.svm, &env.admin, &liquidator.pubkey(), &env.sol_mint);

        // Refresh reserves
        refresh_both(&mut env);

        // Liquidate: repay 40 USDC (50% of 80)
        let liquidate_amount = 40_000_000u64;
        env.svm.expire_blockhash();
        let ix = liquidate_ix(
            &liquidator.pubkey(),
            &user.pubkey(),
            &env.lending_market,
            &env.usdc_reserve,
            &env.sol_reserve,
            &env.sol_reserve_authority,
            &env.usdc_oracle,
            &env.sol_oracle,
            &env.usdc_mint,
            &env.sol_mint,
            &env.usdc_vault,
            &env.sol_vault,
            liquidate_amount,
        );
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&liquidator.pubkey()),
            &[&liquidator],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        // Liquidator should have received SOL with 5% bonus
        let liq_sol = get_associated_token_address(&liquidator.pubkey(), &env.sol_mint);
        let sol_received = token_balance(&env.svm, &liq_sol);
        // Expected: 40 USDC * $1 * 1.05 / $50 = 0.84 SOL = 840_000_000 base units
        println!("Liquidator received {} SOL tokens (expected ~840_000_000)", sol_received);
        assert!(sol_received > 0, "Liquidator should receive collateral");
        // Allow some rounding tolerance
        assert!(
            sol_received >= 839_000_000 && sol_received <= 841_000_000,
            "Expected ~840M, got {}",
            sol_received
        );
    }

    #[test]
    fn test_13_close_factor_enforcement() {
        let mut env = setup();
        let user = Keypair::new();
        let liquidator = Keypair::new();

        let sol_deposit = 1_000_000_000u64;
        let usdc_borrow = 80_000_000u64;

        fund_user(&mut env, &user, sol_deposit, 0);
        env.svm.expire_blockhash();
        create_ata(&mut env.svm, &env.admin, &user.pubkey(), &env.usdc_mint);

        // Seed USDC
        env.svm.expire_blockhash();
        let admin_usdc_ata = create_ata(&mut env.svm, &env.admin, &env.admin.pubkey(), &env.usdc_mint);
        mint_tokens(&mut env.svm, &env.admin, &env.usdc_mint, &admin_usdc_ata, &env.mint_authority, 1_000_000_000_000);
        refresh_both(&mut env);
        env.svm.expire_blockhash();
        let ix = deposit_ix(
            &env.admin.pubkey(),
            &env.lending_market,
            &env.usdc_reserve,
            &env.usdc_mint,
            &env.usdc_vault,
            1_000_000_000_000,
        );
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&env.admin.pubkey()),
            &[&env.admin],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        // Deposit + borrow
        refresh_both(&mut env);
        env.svm.expire_blockhash();
        let ix = deposit_ix(
            &user.pubkey(),
            &env.lending_market,
            &env.sol_reserve,
            &env.sol_mint,
            &env.sol_vault,
            sol_deposit,
        );
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&user.pubkey()),
            &[&user],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        refresh_both(&mut env);
        env.svm.expire_blockhash();
        let ix = borrow_ix(
            &user.pubkey(),
            &env.lending_market,
            &env.usdc_reserve,
            &env.usdc_reserve_authority,
            &env.sol_reserve,
            &env.usdc_oracle,
            &env.sol_oracle,
            &env.usdc_mint,
            &env.usdc_vault,
            usdc_borrow,
        );
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&user.pubkey()),
            &[&user],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        // Drop SOL price
        env.svm.expire_blockhash();
        let ix = update_mock_oracle_ix(&env.admin.pubkey(), &env.sol_mint, 50_000_000);
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&env.admin.pubkey()),
            &[&env.admin],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        // Fund liquidator
        fund_user(&mut env, &liquidator, 0, 1_000_000_000);
        env.svm.expire_blockhash();
        create_ata(&mut env.svm, &env.admin, &liquidator.pubkey(), &env.sol_mint);

        refresh_both(&mut env);

        // Try to liquidate >50% (e.g. 50 USDC out of 80 = 62.5%) -> should fail
        let too_much = 50_000_000u64;
        env.svm.expire_blockhash();
        let ix = liquidate_ix(
            &liquidator.pubkey(),
            &user.pubkey(),
            &env.lending_market,
            &env.usdc_reserve,
            &env.sol_reserve,
            &env.sol_reserve_authority,
            &env.usdc_oracle,
            &env.sol_oracle,
            &env.usdc_mint,
            &env.sol_mint,
            &env.usdc_vault,
            &env.sol_vault,
            too_much,
        );
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&liquidator.pubkey()),
            &[&liquidator],
            env.svm.latest_blockhash(),
        );
        let result = env.svm.send_transaction(tx);
        assert!(result.is_err(), "Should reject: exceeds close factor");
        println!("Close factor correctly enforced");
    }

    #[test]
    fn test_14_liquidation_rejected_healthy() {
        let mut env = setup();
        let user = Keypair::new();
        let liquidator = Keypair::new();

        let sol_deposit = 10_000_000_000u64;
        let usdc_borrow = 500_000_000u64;

        fund_user(&mut env, &user, sol_deposit, 0);
        env.svm.expire_blockhash();
        create_ata(&mut env.svm, &env.admin, &user.pubkey(), &env.usdc_mint);

        env.svm.expire_blockhash();
        let admin_usdc_ata = create_ata(&mut env.svm, &env.admin, &env.admin.pubkey(), &env.usdc_mint);
        mint_tokens(&mut env.svm, &env.admin, &env.usdc_mint, &admin_usdc_ata, &env.mint_authority, 1_000_000_000_000);
        refresh_both(&mut env);
        env.svm.expire_blockhash();
        let ix = deposit_ix(
            &env.admin.pubkey(),
            &env.lending_market,
            &env.usdc_reserve,
            &env.usdc_mint,
            &env.usdc_vault,
            1_000_000_000_000,
        );
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&env.admin.pubkey()),
            &[&env.admin],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        refresh_both(&mut env);
        env.svm.expire_blockhash();
        let ix = deposit_ix(
            &user.pubkey(),
            &env.lending_market,
            &env.sol_reserve,
            &env.sol_mint,
            &env.sol_vault,
            sol_deposit,
        );
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&user.pubkey()),
            &[&user],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        refresh_both(&mut env);
        env.svm.expire_blockhash();
        let ix = borrow_ix(
            &user.pubkey(),
            &env.lending_market,
            &env.usdc_reserve,
            &env.usdc_reserve_authority,
            &env.sol_reserve,
            &env.usdc_oracle,
            &env.sol_oracle,
            &env.usdc_mint,
            &env.usdc_vault,
            usdc_borrow,
        );
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&user.pubkey()),
            &[&user],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        // Fund liquidator
        fund_user(&mut env, &liquidator, 0, 1_000_000_000);
        env.svm.expire_blockhash();
        create_ata(&mut env.svm, &env.admin, &liquidator.pubkey(), &env.sol_mint);

        // Try to liquidate (position is healthy, HF ~1.7)
        refresh_both(&mut env);
        env.svm.expire_blockhash();
        let ix = liquidate_ix(
            &liquidator.pubkey(),
            &user.pubkey(),
            &env.lending_market,
            &env.usdc_reserve,
            &env.sol_reserve,
            &env.sol_reserve_authority,
            &env.usdc_oracle,
            &env.sol_oracle,
            &env.usdc_mint,
            &env.sol_mint,
            &env.usdc_vault,
            &env.sol_vault,
            250_000_000,
        );
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&liquidator.pubkey()),
            &[&liquidator],
            env.svm.latest_blockhash(),
        );
        let result = env.svm.send_transaction(tx);
        assert!(result.is_err(), "Should reject: position is healthy");
        println!("Liquidation correctly rejected for healthy position");
    }

    #[test]
    fn test_15_oracle_staleness() {
        let mut env = setup();
        let user = Keypair::new();
        fund_user(&mut env, &user, 10_000_000_000, 0);

        // Warp far into future so oracle becomes stale (>120s)
        warp_clock(&mut env.svm, 200);

        // Try to refresh reserve -- should fail due to stale oracle
        env.svm.expire_blockhash();
        let ix = refresh_reserve_ix(&env.sol_reserve, &env.sol_oracle);
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&user.pubkey()),
            &[&user],
            env.svm.latest_blockhash(),
        );
        let result = env.svm.send_transaction(tx);
        assert!(result.is_err(), "Should reject: oracle is stale");
        println!("Oracle staleness correctly enforced");
    }

    #[test]
    fn test_16_supply_cap_exceeded() {
        let mut env = setup();

        // Create a reserve with a small supply cap
        // We'll reuse the setup but modify the config by creating a fresh mint
        let tiny_mint = create_mint(&mut env.svm, &env.admin, &env.mint_authority.pubkey(), 6);

        env.svm.expire_blockhash();
        let ix = init_mock_oracle_ix(&env.admin.pubkey(), &tiny_mint, 1_000_000, 6);
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&env.admin.pubkey()),
            &[&env.admin],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        let mut config = usdc_config();
        config.supply_cap = 100_000_000; // 100 tokens

        env.svm.expire_blockhash();
        let ix = init_reserve_ix(&env.admin.pubkey(), &env.lending_market, &tiny_mint, config);
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&env.admin.pubkey()),
            &[&env.admin],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        let (tiny_reserve, _) = reserve_pda(&env.lending_market, &tiny_mint);
        let (tiny_oracle, _) = mock_oracle_pda(&tiny_mint);
        let (tiny_authority, _) = reserve_authority_pda(&tiny_reserve);
        let tiny_vault = get_associated_token_address(&tiny_authority, &tiny_mint);

        // Fund user with more than cap
        let user = Keypair::new();
        env.svm.airdrop(&user.pubkey(), 10_000_000_000).unwrap();
        env.svm.expire_blockhash();
        let ata = create_ata(&mut env.svm, &env.admin, &user.pubkey(), &tiny_mint);
        mint_tokens(&mut env.svm, &env.admin, &tiny_mint, &ata, &env.mint_authority, 200_000_000);

        // Refresh tiny reserve
        env.svm.expire_blockhash();
        let ix = refresh_reserve_ix(&tiny_reserve, &tiny_oracle);
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&user.pubkey()),
            &[&user],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        // Try to deposit 200 (cap is 100)
        env.svm.expire_blockhash();
        let ix = deposit_ix(
            &user.pubkey(),
            &env.lending_market,
            &tiny_reserve,
            &tiny_mint,
            &tiny_vault,
            200_000_000,
        );
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&user.pubkey()),
            &[&user],
            env.svm.latest_blockhash(),
        );
        let result = env.svm.send_transaction(tx);
        assert!(result.is_err(), "Should reject: supply cap exceeded");
        println!("Supply cap correctly enforced");
    }

    #[test]
    fn test_17_borrow_cap_exceeded() {
        let mut env = setup();

        // Create a reserve with small borrow cap
        let tiny_mint = create_mint(&mut env.svm, &env.admin, &env.mint_authority.pubkey(), 6);

        env.svm.expire_blockhash();
        let ix = init_mock_oracle_ix(&env.admin.pubkey(), &tiny_mint, 1_000_000, 6);
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&env.admin.pubkey()),
            &[&env.admin],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        let mut config = usdc_config();
        config.borrow_cap = 10_000_000; // 10 tokens max borrow

        env.svm.expire_blockhash();
        let ix = init_reserve_ix(&env.admin.pubkey(), &env.lending_market, &tiny_mint, config);
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&env.admin.pubkey()),
            &[&env.admin],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        let (tiny_reserve, _) = reserve_pda(&env.lending_market, &tiny_mint);
        let (tiny_oracle, _) = mock_oracle_pda(&tiny_mint);
        let (tiny_authority, _) = reserve_authority_pda(&tiny_reserve);
        let tiny_vault = get_associated_token_address(&tiny_authority, &tiny_mint);

        // Deposit liquidity into tiny reserve
        env.svm.expire_blockhash();
        let admin_ata = create_ata(&mut env.svm, &env.admin, &env.admin.pubkey(), &tiny_mint);
        mint_tokens(&mut env.svm, &env.admin, &tiny_mint, &admin_ata, &env.mint_authority, 1_000_000_000);

        env.svm.expire_blockhash();
        let ix = refresh_reserve_ix(&tiny_reserve, &tiny_oracle);
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&env.admin.pubkey()),
            &[&env.admin],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        env.svm.expire_blockhash();
        let ix = deposit_ix(
            &env.admin.pubkey(),
            &env.lending_market,
            &tiny_reserve,
            &tiny_mint,
            &tiny_vault,
            1_000_000_000,
        );
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&env.admin.pubkey()),
            &[&env.admin],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        // User with SOL collateral tries to borrow 20 tokens (cap is 10)
        let user = Keypair::new();
        fund_user(&mut env, &user, 10_000_000_000, 0);
        env.svm.expire_blockhash();
        create_ata(&mut env.svm, &env.admin, &user.pubkey(), &tiny_mint);

        refresh_both(&mut env);
        // Also refresh tiny reserve
        env.svm.expire_blockhash();
        let ix = refresh_reserve_ix(&tiny_reserve, &tiny_oracle);
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&user.pubkey()),
            &[&user],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        // Deposit SOL
        env.svm.expire_blockhash();
        let ix = deposit_ix(
            &user.pubkey(),
            &env.lending_market,
            &env.sol_reserve,
            &env.sol_mint,
            &env.sol_vault,
            10_000_000_000,
        );
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&user.pubkey()),
            &[&user],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        // Refresh all (sol + tiny)
        env.svm.expire_blockhash();
        let ix1 = refresh_reserve_ix(&env.sol_reserve, &env.sol_oracle);
        let ix2 = refresh_reserve_ix(&tiny_reserve, &tiny_oracle);
        let tx = Transaction::new_signed_with_payer(
            &[ix1, ix2],
            Some(&user.pubkey()),
            &[&user],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        // Try borrow 20 tokens
        env.svm.expire_blockhash();
        let ix = borrow_ix(
            &user.pubkey(),
            &env.lending_market,
            &tiny_reserve,
            &tiny_authority,
            &env.sol_reserve,
            &tiny_oracle,
            &env.sol_oracle,
            &tiny_mint,
            &tiny_vault,
            20_000_000, // 20 tokens, cap is 10
        );
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&user.pubkey()),
            &[&user],
            env.svm.latest_blockhash(),
        );
        let result = env.svm.send_transaction(tx);
        assert!(result.is_err(), "Should reject: borrow cap exceeded");
        println!("Borrow cap correctly enforced");
    }

    #[test]
    fn test_18_full_lifecycle() {
        let mut env = setup();
        let user = Keypair::new();
        let liquidator = Keypair::new();

        // ── Phase 1: Setup ──
        let sol_deposit = 10_000_000_000u64; // 10 SOL ($1000)
        let usdc_borrow = 500_000_000u64;    // 500 USDC

        fund_user(&mut env, &user, sol_deposit, 1_000_000_000); // extra for repay

        // Seed USDC liquidity
        env.svm.expire_blockhash();
        let admin_usdc_ata = create_ata(&mut env.svm, &env.admin, &env.admin.pubkey(), &env.usdc_mint);
        mint_tokens(&mut env.svm, &env.admin, &env.usdc_mint, &admin_usdc_ata, &env.mint_authority, 1_000_000_000_000);
        refresh_both(&mut env);
        env.svm.expire_blockhash();
        let ix = deposit_ix(
            &env.admin.pubkey(),
            &env.lending_market,
            &env.usdc_reserve,
            &env.usdc_mint,
            &env.usdc_vault,
            1_000_000_000_000,
        );
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&env.admin.pubkey()),
            &[&env.admin],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        // ── Phase 2: Deposit + Borrow ──
        refresh_both(&mut env);
        env.svm.expire_blockhash();
        let ix = deposit_ix(
            &user.pubkey(),
            &env.lending_market,
            &env.sol_reserve,
            &env.sol_mint,
            &env.sol_vault,
            sol_deposit,
        );
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&user.pubkey()),
            &[&user],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        refresh_both(&mut env);
        env.svm.expire_blockhash();
        let ix = borrow_ix(
            &user.pubkey(),
            &env.lending_market,
            &env.usdc_reserve,
            &env.usdc_reserve_authority,
            &env.sol_reserve,
            &env.usdc_oracle,
            &env.sol_oracle,
            &env.usdc_mint,
            &env.usdc_vault,
            usdc_borrow,
        );
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&user.pubkey()),
            &[&user],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();
        println!("Phase 2: Deposited {} SOL, borrowed {} USDC", sol_deposit, usdc_borrow);

        // ── Phase 3: Warp time, accrue interest ──
        warp_clock(&mut env.svm, 30 * 24 * 3600); // 30 days
        env.svm.expire_blockhash();
        let ix1 = update_mock_oracle_ix(&env.admin.pubkey(), &env.sol_mint, SOL_PRICE);
        let ix2 = update_mock_oracle_ix(&env.admin.pubkey(), &env.usdc_mint, USDC_PRICE);
        let tx = Transaction::new_signed_with_payer(
            &[ix1, ix2],
            Some(&env.admin.pubkey()),
            &[&env.admin],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        refresh_both(&mut env);

        let reserve_data = env.svm.get_account(&env.usdc_reserve).unwrap();
        let reserve: klend::state::Reserve =
            anchor_lang::AccountDeserialize::try_deserialize(&mut &reserve_data.data[..]).unwrap();
        println!(
            "Phase 3: After 30 days - borrowed: {}, fees: {}",
            reserve.borrowed_liquidity, reserve.accumulated_protocol_fees
        );
        assert!(reserve.borrowed_liquidity > usdc_borrow);

        // ── Phase 4: Partial repay ──
        let repay_amount = 250_000_000u64;
        env.svm.expire_blockhash();
        let ix = repay_ix(
            &user.pubkey(),
            &user.pubkey(),
            &env.lending_market,
            &env.usdc_reserve,
            &env.usdc_mint,
            &env.usdc_vault,
            repay_amount,
        );
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&user.pubkey()),
            &[&user],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();
        println!("Phase 4: Repaid {} USDC", repay_amount);

        // ── Phase 5: Price drop + liquidation ──
        env.svm.expire_blockhash();
        let ix = update_mock_oracle_ix(&env.admin.pubkey(), &env.sol_mint, 20_000_000); // $20
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&env.admin.pubkey()),
            &[&env.admin],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        fund_user(&mut env, &liquidator, 0, 1_000_000_000);
        env.svm.expire_blockhash();
        create_ata(&mut env.svm, &env.admin, &liquidator.pubkey(), &env.sol_mint);

        refresh_both(&mut env);

        // Read current debt to compute valid liquidation amount
        let reserve_data = env.svm.get_account(&env.usdc_reserve).unwrap();
        let reserve: klend::state::Reserve =
            anchor_lang::AccountDeserialize::try_deserialize(&mut &reserve_data.data[..]).unwrap();

        let obligation_key = obligation_pda(&env.lending_market, &user.pubkey()).0;
        let ob_data = env.svm.get_account(&obligation_key).unwrap();
        let obligation: klend::state::Obligation =
            anchor_lang::AccountDeserialize::try_deserialize(&mut &ob_data.data[..]).unwrap();

        let borrow = obligation
            .borrows
            .iter()
            .find(|b| b.reserve == env.usdc_reserve)
            .unwrap();
        let current_debt = borrow.current_debt(reserve.cumulative_borrow_index);
        let max_liquidate = current_debt / 2; // 50% close factor
        println!("Phase 5: Current debt: {}, max liquidation: {}", current_debt, max_liquidate);

        env.svm.expire_blockhash();
        let ix = liquidate_ix(
            &liquidator.pubkey(),
            &user.pubkey(),
            &env.lending_market,
            &env.usdc_reserve,
            &env.sol_reserve,
            &env.sol_reserve_authority,
            &env.usdc_oracle,
            &env.sol_oracle,
            &env.usdc_mint,
            &env.sol_mint,
            &env.usdc_vault,
            &env.sol_vault,
            max_liquidate,
        );
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&liquidator.pubkey()),
            &[&liquidator],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        let liq_sol = get_associated_token_address(&liquidator.pubkey(), &env.sol_mint);
        let sol_received = token_balance(&env.svm, &liq_sol);
        println!("Phase 5: Liquidator received {} SOL tokens", sol_received);
        assert!(sol_received > 0);
        println!("Full lifecycle test passed!");
    }
}
