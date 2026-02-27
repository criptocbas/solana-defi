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

    use kusd::constants::*;

    const PROGRAM_ID: &str = "4niQV7cydNxRakwi4hM2jhSkp6dwg4abuzx5HsAwDz95";

    fn program_id() -> Pubkey {
        Pubkey::from_str(PROGRAM_ID).unwrap()
    }

    // ── PDA helpers ──────────────────────────────────────────────

    fn cdp_vault_pda(collateral_mint: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[CDP_VAULT_SEED, collateral_mint.as_ref()],
            &program_id(),
        )
    }

    fn vault_authority_pda(vault: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[CDP_VAULT_AUTHORITY_SEED, vault.as_ref()],
            &program_id(),
        )
    }

    fn kusd_mint_pda(vault: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[KUSD_MINT_SEED, vault.as_ref()], &program_id())
    }

    fn cdp_position_pda(vault: &Pubkey, owner: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[CDP_POSITION_SEED, vault.as_ref(), owner.as_ref()],
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

    // ── Test constants ──────────────────────────────────────────

    const SOL_PRICE: u64 = 100_000_000; // $100 * 1e6
    const SOL_DECIMALS: u8 = 9;
    const MAX_LTV_BPS: u16 = 6700;      // 67%
    const LIQ_THRESHOLD_BPS: u16 = 7500; // 75%
    const LIQ_BONUS_BPS: u16 = 500;     // 5%
    const STABILITY_FEE_BPS: u16 = 200; // 2% annual
    const DEBT_CEILING: u64 = 1_000_000_000_000; // 1M kUSD (6 decimals)
    const ORACLE_MAX_STALENESS: u64 = 120;

    // ── TestEnv ──────────────────────────────────────────────────

    struct TestEnv {
        svm: LiteSVM,
        admin: Keypair,
        mint_authority: Keypair,
        collateral_mint: Pubkey,
        vault: Pubkey,
        vault_authority: Pubkey,
        kusd_mint: Pubkey,
        collateral_vault: Pubkey,
        oracle: Pubkey,
    }

    fn setup() -> TestEnv {
        let mut svm = LiteSVM::new();
        let program_bytes = include_bytes!("../../target/deploy/kusd.so");
        svm.add_program(program_id(), program_bytes).unwrap();

        let admin = Keypair::new();
        svm.airdrop(&admin.pubkey(), 100_000_000_000).unwrap();

        let mint_authority = Keypair::new();
        let collateral_mint = create_mint(&mut svm, &admin, &mint_authority.pubkey(), SOL_DECIMALS);

        // Create oracle
        let (oracle, _) = mock_oracle_pda(&collateral_mint);
        let ix = Instruction {
            program_id: program_id(),
            accounts: kusd::accounts::InitMockOracle {
                payer: admin.pubkey(),
                token_mint: collateral_mint,
                oracle,
                system_program: system_program::ID,
            }
            .to_account_metas(None),
            data: kusd::instruction::InitMockOracle {
                price: SOL_PRICE,
                decimals: SOL_DECIMALS,
            }
            .data(),
        };
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&admin.pubkey()),
            &[&admin],
            svm.latest_blockhash(),
        );
        svm.send_transaction(tx).unwrap();

        // Init vault
        let (vault, _) = cdp_vault_pda(&collateral_mint);
        let (vault_authority, _) = vault_authority_pda(&vault);
        let (kusd_mint, _) = kusd_mint_pda(&vault);
        let collateral_vault = get_associated_token_address(&vault_authority, &collateral_mint);

        let ix = Instruction {
            program_id: program_id(),
            accounts: kusd::accounts::InitVault {
                admin: admin.pubkey(),
                collateral_mint,
                oracle,
                vault,
                vault_authority,
                kusd_mint,
                collateral_token_account: collateral_vault,
                token_program: spl_token::id(),
                associated_token_program: spl_associated_token_account::id(),
                system_program: system_program::ID,
            }
            .to_account_metas(None),
            data: kusd::instruction::InitVault {
                max_ltv_bps: MAX_LTV_BPS,
                liquidation_threshold_bps: LIQ_THRESHOLD_BPS,
                liquidation_bonus_bps: LIQ_BONUS_BPS,
                stability_fee_bps: STABILITY_FEE_BPS,
                debt_ceiling: DEBT_CEILING,
                oracle_max_staleness: ORACLE_MAX_STALENESS,
            }
            .data(),
        };
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&admin.pubkey()),
            &[&admin],
            svm.latest_blockhash(),
        );
        svm.send_transaction(tx).unwrap();

        TestEnv {
            svm,
            admin,
            mint_authority,
            collateral_mint,
            vault,
            vault_authority,
            kusd_mint,
            collateral_vault,
            oracle,
        }
    }

    /// Create a user with collateral tokens and a kUSD ATA, open position
    fn create_user(env: &mut TestEnv, collateral_amount: u64) -> (Keypair, Pubkey, Pubkey) {
        let user = Keypair::new();
        env.svm.airdrop(&user.pubkey(), 10_000_000_000).unwrap();

        // Create user's collateral ATA and mint tokens
        let user_collateral = create_ata(&mut env.svm, &user, &user.pubkey(), &env.collateral_mint);
        if collateral_amount > 0 {
            mint_tokens(
                &mut env.svm,
                &env.admin,
                &env.collateral_mint,
                &user_collateral,
                &env.mint_authority,
                collateral_amount,
            );
        }

        // Create user's kUSD ATA
        let user_kusd = create_ata(&mut env.svm, &user, &user.pubkey(), &env.kusd_mint);

        // Open position
        let (position, _) = cdp_position_pda(&env.vault, &user.pubkey());
        let ix = Instruction {
            program_id: program_id(),
            accounts: kusd::accounts::OpenPosition {
                owner: user.pubkey(),
                vault: env.vault,
                position,
                system_program: system_program::ID,
            }
            .to_account_metas(None),
            data: kusd::instruction::OpenPosition {}.data(),
        };
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&user.pubkey()),
            &[&user],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        (user, user_collateral, user_kusd)
    }

    fn deposit(env: &mut TestEnv, user: &Keypair, user_collateral: &Pubkey, amount: u64) {
        let (position, _) = cdp_position_pda(&env.vault, &user.pubkey());
        let ix = Instruction {
            program_id: program_id(),
            accounts: kusd::accounts::DepositCollateral {
                depositor: user.pubkey(),
                vault: env.vault,
                position,
                owner: user.pubkey(),
                depositor_collateral: *user_collateral,
                collateral_vault: env.collateral_vault,
                token_program: spl_token::id(),
            }
            .to_account_metas(None),
            data: kusd::instruction::DepositCollateral { amount }.data(),
        };
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&user.pubkey()),
            &[user],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();
        env.svm.expire_blockhash();
    }

    fn mint_kusd(env: &mut TestEnv, user: &Keypair, user_kusd: &Pubkey, amount: u64) {
        let (position, _) = cdp_position_pda(&env.vault, &user.pubkey());
        let ix = Instruction {
            program_id: program_id(),
            accounts: kusd::accounts::MintKusd {
                borrower: user.pubkey(),
                vault: env.vault,
                vault_authority: env.vault_authority,
                position,
                owner: user.pubkey(),
                kusd_mint: env.kusd_mint,
                borrower_kusd: *user_kusd,
                oracle: env.oracle,
                token_program: spl_token::id(),
            }
            .to_account_metas(None),
            data: kusd::instruction::MintKusd { amount }.data(),
        };
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&user.pubkey()),
            &[user],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();
        env.svm.expire_blockhash();
    }

    fn try_mint_kusd(
        env: &mut TestEnv,
        user: &Keypair,
        user_kusd: &Pubkey,
        amount: u64,
    ) -> Result<(), litesvm::types::FailedTransactionMetadata> {
        let (position, _) = cdp_position_pda(&env.vault, &user.pubkey());
        let ix = Instruction {
            program_id: program_id(),
            accounts: kusd::accounts::MintKusd {
                borrower: user.pubkey(),
                vault: env.vault,
                vault_authority: env.vault_authority,
                position,
                owner: user.pubkey(),
                kusd_mint: env.kusd_mint,
                borrower_kusd: *user_kusd,
                oracle: env.oracle,
                token_program: spl_token::id(),
            }
            .to_account_metas(None),
            data: kusd::instruction::MintKusd { amount }.data(),
        };
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&user.pubkey()),
            &[user],
            env.svm.latest_blockhash(),
        );
        let result = env.svm.send_transaction(tx).map(|_| ());
        env.svm.expire_blockhash();
        result
    }

    fn repay(env: &mut TestEnv, payer: &Keypair, position_owner: &Pubkey, payer_kusd: &Pubkey, amount: u64) {
        let (position, _) = cdp_position_pda(&env.vault, position_owner);
        let ix = Instruction {
            program_id: program_id(),
            accounts: kusd::accounts::RepayKusd {
                payer: payer.pubkey(),
                vault: env.vault,
                position,
                position_owner: *position_owner,
                kusd_mint: env.kusd_mint,
                payer_kusd: *payer_kusd,
                token_program: spl_token::id(),
            }
            .to_account_metas(None),
            data: kusd::instruction::RepayKusd { amount }.data(),
        };
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&payer.pubkey()),
            &[payer],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();
        env.svm.expire_blockhash();
    }

    fn withdraw(env: &mut TestEnv, user: &Keypair, user_collateral: &Pubkey, amount: u64) {
        let (position, _) = cdp_position_pda(&env.vault, &user.pubkey());
        let ix = Instruction {
            program_id: program_id(),
            accounts: kusd::accounts::WithdrawCollateral {
                owner: user.pubkey(),
                vault: env.vault,
                vault_authority: env.vault_authority,
                position,
                owner_collateral: *user_collateral,
                collateral_vault: env.collateral_vault,
                oracle: env.oracle,
                token_program: spl_token::id(),
            }
            .to_account_metas(None),
            data: kusd::instruction::WithdrawCollateral { amount }.data(),
        };
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&user.pubkey()),
            &[user],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();
        env.svm.expire_blockhash();
    }

    fn try_withdraw(
        env: &mut TestEnv,
        user: &Keypair,
        user_collateral: &Pubkey,
        amount: u64,
    ) -> Result<(), litesvm::types::FailedTransactionMetadata> {
        let (position, _) = cdp_position_pda(&env.vault, &user.pubkey());
        let ix = Instruction {
            program_id: program_id(),
            accounts: kusd::accounts::WithdrawCollateral {
                owner: user.pubkey(),
                vault: env.vault,
                vault_authority: env.vault_authority,
                position,
                owner_collateral: *user_collateral,
                collateral_vault: env.collateral_vault,
                oracle: env.oracle,
                token_program: spl_token::id(),
            }
            .to_account_metas(None),
            data: kusd::instruction::WithdrawCollateral { amount }.data(),
        };
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&user.pubkey()),
            &[user],
            env.svm.latest_blockhash(),
        );
        let result = env.svm.send_transaction(tx).map(|_| ());
        env.svm.expire_blockhash();
        result
    }

    fn liquidate_position(
        env: &mut TestEnv,
        liquidator: &Keypair,
        position_owner: &Pubkey,
        liquidator_kusd: &Pubkey,
        liquidator_collateral: &Pubkey,
        repay_amount: u64,
    ) {
        let (position, _) = cdp_position_pda(&env.vault, position_owner);
        let ix = Instruction {
            program_id: program_id(),
            accounts: kusd::accounts::Liquidate {
                liquidator: liquidator.pubkey(),
                vault: env.vault,
                vault_authority: env.vault_authority,
                position,
                position_owner: *position_owner,
                oracle: env.oracle,
                kusd_mint: env.kusd_mint,
                liquidator_kusd: *liquidator_kusd,
                liquidator_collateral: *liquidator_collateral,
                collateral_vault: env.collateral_vault,
                token_program: spl_token::id(),
            }
            .to_account_metas(None),
            data: kusd::instruction::Liquidate { repay_amount }.data(),
        };
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&liquidator.pubkey()),
            &[liquidator],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();
        env.svm.expire_blockhash();
    }

    fn try_liquidate(
        env: &mut TestEnv,
        liquidator: &Keypair,
        position_owner: &Pubkey,
        liquidator_kusd: &Pubkey,
        liquidator_collateral: &Pubkey,
        repay_amount: u64,
    ) -> Result<(), litesvm::types::FailedTransactionMetadata> {
        let (position, _) = cdp_position_pda(&env.vault, position_owner);
        let ix = Instruction {
            program_id: program_id(),
            accounts: kusd::accounts::Liquidate {
                liquidator: liquidator.pubkey(),
                vault: env.vault,
                vault_authority: env.vault_authority,
                position,
                position_owner: *position_owner,
                oracle: env.oracle,
                kusd_mint: env.kusd_mint,
                liquidator_kusd: *liquidator_kusd,
                liquidator_collateral: *liquidator_collateral,
                collateral_vault: env.collateral_vault,
                token_program: spl_token::id(),
            }
            .to_account_metas(None),
            data: kusd::instruction::Liquidate { repay_amount }.data(),
        };
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&liquidator.pubkey()),
            &[liquidator],
            env.svm.latest_blockhash(),
        );
        let result = env.svm.send_transaction(tx).map(|_| ());
        env.svm.expire_blockhash();
        result
    }

    fn update_oracle(env: &mut TestEnv, price: u64) {
        let ix = Instruction {
            program_id: program_id(),
            accounts: kusd::accounts::UpdateMockOracle {
                payer: env.admin.pubkey(),
                oracle: env.oracle,
            }
            .to_account_metas(None),
            data: kusd::instruction::UpdateMockOracle { price }.data(),
        };
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&env.admin.pubkey()),
            &[&env.admin],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();
        env.svm.expire_blockhash();
    }

    fn accrue_fees_ix(env: &mut TestEnv) {
        let ix = Instruction {
            program_id: program_id(),
            accounts: kusd::accounts::AccrueFees {
                vault: env.vault,
            }
            .to_account_metas(None),
            data: kusd::instruction::AccrueFees {}.data(),
        };
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&env.admin.pubkey()),
            &[&env.admin],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();
        env.svm.expire_blockhash();
    }

    fn read_vault(env: &TestEnv) -> kusd::state::CdpVault {
        let data = env.svm.get_account(&env.vault).unwrap();
        let mut slice = &data.data[8..]; // skip discriminator
        anchor_lang::AnchorDeserialize::deserialize(&mut slice).unwrap()
    }

    fn read_position(env: &TestEnv, owner: &Pubkey) -> kusd::state::CdpPosition {
        let (position, _) = cdp_position_pda(&env.vault, owner);
        let data = env.svm.get_account(&position).unwrap();
        let mut slice = &data.data[8..];
        anchor_lang::AnchorDeserialize::deserialize(&mut slice).unwrap()
    }

    // ── Tests ────────────────────────────────────────────────────

    // 1. Init vault basic
    #[test]
    fn test_init_vault_basic() {
        let env = setup();
        let vault = read_vault(&env);

        assert_eq!(vault.admin, env.admin.pubkey());
        assert_eq!(vault.collateral_mint, env.collateral_mint);
        assert_eq!(vault.max_ltv_bps, MAX_LTV_BPS);
        assert_eq!(vault.liquidation_threshold_bps, LIQ_THRESHOLD_BPS);
        assert_eq!(vault.liquidation_bonus_bps, LIQ_BONUS_BPS);
        assert_eq!(vault.stability_fee_bps, STABILITY_FEE_BPS);
        assert_eq!(vault.debt_ceiling, DEBT_CEILING);
        assert_eq!(vault.cumulative_fee_index, SCALE);
        assert_eq!(vault.total_collateral, 0);
        assert_eq!(vault.total_debt_shares, 0);
        assert!(!vault.halted);
        assert_eq!(vault.collateral_decimals, SOL_DECIMALS);
    }

    // 2. Init vault invalid config
    #[test]
    fn test_init_vault_invalid_config() {
        let mut svm = LiteSVM::new();
        let program_bytes = include_bytes!("../../target/deploy/kusd.so");
        svm.add_program(program_id(), program_bytes).unwrap();

        let admin = Keypair::new();
        svm.airdrop(&admin.pubkey(), 100_000_000_000).unwrap();

        let mint_auth = Keypair::new();
        let collateral_mint = create_mint(&mut svm, &admin, &mint_auth.pubkey(), SOL_DECIMALS);

        // Create oracle
        let (oracle, _) = mock_oracle_pda(&collateral_mint);
        let ix = Instruction {
            program_id: program_id(),
            accounts: kusd::accounts::InitMockOracle {
                payer: admin.pubkey(),
                token_mint: collateral_mint,
                oracle,
                system_program: system_program::ID,
            }
            .to_account_metas(None),
            data: kusd::instruction::InitMockOracle {
                price: SOL_PRICE,
                decimals: SOL_DECIMALS,
            }
            .data(),
        };
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&admin.pubkey()),
            &[&admin],
            svm.latest_blockhash(),
        );
        svm.send_transaction(tx).unwrap();

        // Try init vault with max_ltv >= liq_threshold
        let (vault, _) = cdp_vault_pda(&collateral_mint);
        let (vault_authority, _) = vault_authority_pda(&vault);
        let (kusd_mint, _) = kusd_mint_pda(&vault);
        let collateral_vault_ata = get_associated_token_address(&vault_authority, &collateral_mint);

        let ix = Instruction {
            program_id: program_id(),
            accounts: kusd::accounts::InitVault {
                admin: admin.pubkey(),
                collateral_mint,
                oracle,
                vault,
                vault_authority,
                kusd_mint,
                collateral_token_account: collateral_vault_ata,
                token_program: spl_token::id(),
                associated_token_program: spl_associated_token_account::id(),
                system_program: system_program::ID,
            }
            .to_account_metas(None),
            data: kusd::instruction::InitVault {
                max_ltv_bps: 8000,               // 80%
                liquidation_threshold_bps: 7500,  // 75% < max_ltv! Invalid
                liquidation_bonus_bps: 500,
                stability_fee_bps: 200,
                debt_ceiling: 0,
                oracle_max_staleness: 120,
            }
            .data(),
        };
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&admin.pubkey()),
            &[&admin],
            svm.latest_blockhash(),
        );
        let result = svm.send_transaction(tx);
        assert!(result.is_err());
    }

    // 3. Open position
    #[test]
    fn test_open_position() {
        let mut env = setup();
        let (user, _, _) = create_user(&mut env, 0);

        let pos = read_position(&env, &user.pubkey());
        assert_eq!(pos.vault, env.vault);
        assert_eq!(pos.owner, user.pubkey());
        assert_eq!(pos.collateral_amount, 0);
        assert_eq!(pos.debt_shares, 0);
    }

    // 4. Deposit collateral
    #[test]
    fn test_deposit_collateral() {
        let mut env = setup();
        let deposit_amount = 10_000_000_000u64; // 10 SOL
        let (user, user_collateral, _) = create_user(&mut env, deposit_amount);

        deposit(&mut env, &user, &user_collateral, deposit_amount);

        let pos = read_position(&env, &user.pubkey());
        assert_eq!(pos.collateral_amount, deposit_amount);

        let vault = read_vault(&env);
        assert_eq!(vault.total_collateral, deposit_amount);

        assert_eq!(token_balance(&env.svm, &user_collateral), 0);
        assert_eq!(token_balance(&env.svm, &env.collateral_vault), deposit_amount);
    }

    // 5. Deposit multiple
    #[test]
    fn test_deposit_multiple() {
        let mut env = setup();
        let total = 20_000_000_000u64; // 20 SOL
        let (user, user_collateral, _) = create_user(&mut env, total);

        deposit(&mut env, &user, &user_collateral, 10_000_000_000);
        deposit(&mut env, &user, &user_collateral, 10_000_000_000);

        let pos = read_position(&env, &user.pubkey());
        assert_eq!(pos.collateral_amount, total);

        let vault = read_vault(&env);
        assert_eq!(vault.total_collateral, total);
    }

    // 6. Mint kUSD basic
    #[test]
    fn test_mint_kusd_basic() {
        let mut env = setup();
        let deposit_amount = 10_000_000_000u64; // 10 SOL = $1000
        let (user, user_collateral, user_kusd) = create_user(&mut env, deposit_amount);

        deposit(&mut env, &user, &user_collateral, deposit_amount);

        // Mint 500 kUSD (50% LTV, well under 67% max)
        let mint_amount = 500_000_000u64; // 500 kUSD
        mint_kusd(&mut env, &user, &user_kusd, mint_amount);

        assert_eq!(token_balance(&env.svm, &user_kusd), mint_amount);

        let pos = read_position(&env, &user.pubkey());
        assert!(pos.debt_shares > 0);

        let vault = read_vault(&env);
        assert!(vault.total_debt_shares > 0);
    }

    // 7. Mint kUSD max LTV
    #[test]
    fn test_mint_kusd_max_ltv() {
        let mut env = setup();
        let deposit_amount = 10_000_000_000u64; // 10 SOL = $1000
        let (user, user_collateral, user_kusd) = create_user(&mut env, deposit_amount);

        deposit(&mut env, &user, &user_collateral, deposit_amount);

        // 10 SOL * $100 * 67% = $670 max kUSD
        // Try 700 kUSD (70% LTV) → should fail
        let result = try_mint_kusd(&mut env, &user, &user_kusd, 700_000_000);
        assert!(result.is_err());

        // 670 kUSD (67% LTV) → should succeed
        mint_kusd(&mut env, &user, &user_kusd, 670_000_000);
        assert_eq!(token_balance(&env.svm, &user_kusd), 670_000_000);
    }

    // 8. Mint kUSD debt ceiling
    #[test]
    fn test_mint_kusd_debt_ceiling() {
        let mut env = setup();
        // Need enough collateral for the debt ceiling
        // $1M kUSD / 67% LTV = ~$1.49M collateral = ~14,925 SOL
        let deposit_amount = 15_000_000_000_000u64; // 15,000 SOL
        let (user, user_collateral, user_kusd) = create_user(&mut env, deposit_amount);

        deposit(&mut env, &user, &user_collateral, deposit_amount);

        // Mint exactly at ceiling (1M kUSD)
        mint_kusd(&mut env, &user, &user_kusd, DEBT_CEILING as u64);

        // Try to mint 1 more → should fail
        let result = try_mint_kusd(&mut env, &user, &user_kusd, 1);
        assert!(result.is_err());
    }

    // 9. Mint kUSD halted
    #[test]
    fn test_mint_kusd_halted() {
        let mut env = setup();
        let deposit_amount = 10_000_000_000u64;
        let (user, user_collateral, user_kusd) = create_user(&mut env, deposit_amount);

        deposit(&mut env, &user, &user_collateral, deposit_amount);

        // Halt vault
        let ix = Instruction {
            program_id: program_id(),
            accounts: kusd::accounts::SetHalt {
                admin: env.admin.pubkey(),
                vault: env.vault,
            }
            .to_account_metas(None),
            data: kusd::instruction::SetHalt { halted: true }.data(),
        };
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&env.admin.pubkey()),
            &[&env.admin],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        // Try mint → VaultHalted error
        let result = try_mint_kusd(&mut env, &user, &user_kusd, 100_000_000);
        assert!(result.is_err());
    }

    // 10. Repay partial
    #[test]
    fn test_repay_partial() {
        let mut env = setup();
        let deposit_amount = 10_000_000_000u64;
        let mint_amount = 500_000_000u64;
        let (user, user_collateral, user_kusd) = create_user(&mut env, deposit_amount);

        deposit(&mut env, &user, &user_collateral, deposit_amount);
        mint_kusd(&mut env, &user, &user_kusd, mint_amount);

        let shares_before = read_position(&env, &user.pubkey()).debt_shares;

        // Repay half
        repay(&mut env, &user, &user.pubkey(), &user_kusd, 250_000_000);

        let pos = read_position(&env, &user.pubkey());
        assert!(pos.debt_shares < shares_before);
        assert!(pos.debt_shares > 0);
        assert_eq!(token_balance(&env.svm, &user_kusd), 250_000_000);
    }

    // 11. Repay full
    #[test]
    fn test_repay_full() {
        let mut env = setup();
        let deposit_amount = 10_000_000_000u64;
        let mint_amount = 500_000_000u64;
        let (user, user_collateral, user_kusd) = create_user(&mut env, deposit_amount);

        deposit(&mut env, &user, &user_collateral, deposit_amount);
        mint_kusd(&mut env, &user, &user_kusd, mint_amount);

        // Repay full (pass u64::MAX to repay all)
        repay(&mut env, &user, &user.pubkey(), &user_kusd, u64::MAX);

        let pos = read_position(&env, &user.pubkey());
        assert_eq!(pos.debt_shares, 0);
        assert_eq!(token_balance(&env.svm, &user_kusd), 0);
    }

    // 12. Withdraw basic (no debt)
    #[test]
    fn test_withdraw_basic() {
        let mut env = setup();
        let deposit_amount = 10_000_000_000u64;
        let (user, user_collateral, _) = create_user(&mut env, deposit_amount);

        deposit(&mut env, &user, &user_collateral, deposit_amount);
        withdraw(&mut env, &user, &user_collateral, deposit_amount);

        let pos = read_position(&env, &user.pubkey());
        assert_eq!(pos.collateral_amount, 0);
        assert_eq!(token_balance(&env.svm, &user_collateral), deposit_amount);
    }

    // 13. Withdraw blocked when undercollateralized
    #[test]
    fn test_withdraw_blocked_undercollateralized() {
        let mut env = setup();
        let deposit_amount = 10_000_000_000u64; // 10 SOL = $1000
        let (user, user_collateral, user_kusd) = create_user(&mut env, deposit_amount);

        deposit(&mut env, &user, &user_collateral, deposit_amount);
        // Mint at max LTV: 670 kUSD
        mint_kusd(&mut env, &user, &user_kusd, 670_000_000);

        // Try to withdraw any collateral → should fail (already at max LTV)
        let result = try_withdraw(&mut env, &user, &user_collateral, 1);
        assert!(result.is_err());
    }

    // 14. Liquidate basic
    #[test]
    fn test_liquidate_basic() {
        let mut env = setup();
        let deposit_amount = 10_000_000_000u64; // 10 SOL = $1000
        let (user, user_collateral, user_kusd) = create_user(&mut env, deposit_amount);

        deposit(&mut env, &user, &user_collateral, deposit_amount);
        // Mint 670 kUSD (67% LTV, right at max)
        mint_kusd(&mut env, &user, &user_kusd, 670_000_000);

        // Create liquidator with kUSD
        let liquidator = Keypair::new();
        env.svm.airdrop(&liquidator.pubkey(), 10_000_000_000).unwrap();
        let liq_kusd = create_ata(&mut env.svm, &liquidator, &liquidator.pubkey(), &env.kusd_mint);
        let liq_collateral = create_ata(&mut env.svm, &liquidator, &liquidator.pubkey(), &env.collateral_mint);

        // Transfer some kUSD to liquidator
        let transfer_ix = spl_token::instruction::transfer(
            &spl_token::id(),
            &user_kusd,
            &liq_kusd,
            &user.pubkey(),
            &[],
            335_000_000, // 335 kUSD (50% of 670)
        )
        .unwrap();
        let tx = Transaction::new_signed_with_payer(
            &[transfer_ix],
            Some(&user.pubkey()),
            &[&user],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        // Drop SOL price from $100 → $80 to make position unhealthy
        // At $80: collateral = $800, debt = $670
        // HF = $800 * 75% / $670 = $600 / $670 = 0.895 < 1.0
        update_oracle(&mut env, 80_000_000);

        let liq_coll_before = token_balance(&env.svm, &liq_collateral);

        // Liquidate 50% of debt = 335 kUSD
        liquidate_position(
            &mut env,
            &liquidator,
            &user.pubkey(),
            &liq_kusd,
            &liq_collateral,
            335_000_000,
        );

        // Liquidator should have received collateral with bonus
        // seized = 335_000_000 * (10000 + 500) * 10^9 / (10000 * 80_000_000)
        // = 335_000_000 * 10500 * 1_000_000_000 / (10000 * 80_000_000)
        // = 335_000_000 * 10500 / 80 * 100 (simplified)
        // = 4_396_875_000  (≈4.397 SOL)
        let liq_coll_after = token_balance(&env.svm, &liq_collateral);
        let seized = liq_coll_after - liq_coll_before;
        assert!(seized > 0, "Liquidator should receive collateral");

        // Expected: 335e6 * 10500 * 1e9 / (10000 * 80e6) = 4_396_875_000
        assert_eq!(seized, 4_396_875_000);

        // Liquidator's kUSD should be burned
        assert_eq!(token_balance(&env.svm, &liq_kusd), 0);
    }

    // 15. Liquidate close factor enforcement
    #[test]
    fn test_liquidate_close_factor() {
        let mut env = setup();
        let deposit_amount = 10_000_000_000u64;
        let (user, user_collateral, user_kusd) = create_user(&mut env, deposit_amount);

        deposit(&mut env, &user, &user_collateral, deposit_amount);
        mint_kusd(&mut env, &user, &user_kusd, 670_000_000);

        // Create liquidator
        let liquidator = Keypair::new();
        env.svm.airdrop(&liquidator.pubkey(), 10_000_000_000).unwrap();
        let liq_kusd = create_ata(&mut env.svm, &liquidator, &liquidator.pubkey(), &env.kusd_mint);
        let liq_collateral = create_ata(&mut env.svm, &liquidator, &liquidator.pubkey(), &env.collateral_mint);

        // Transfer kUSD to liquidator
        let transfer_ix = spl_token::instruction::transfer(
            &spl_token::id(),
            &user_kusd,
            &liq_kusd,
            &user.pubkey(),
            &[],
            670_000_000,
        )
        .unwrap();
        let tx = Transaction::new_signed_with_payer(
            &[transfer_ix],
            Some(&user.pubkey()),
            &[&user],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        // Drop price
        update_oracle(&mut env, 80_000_000);

        // Try to liquidate >50% (336 kUSD > 335 = 50% of 670) → should fail
        let result = try_liquidate(
            &mut env,
            &liquidator,
            &user.pubkey(),
            &liq_kusd,
            &liq_collateral,
            336_000_000,
        );
        assert!(result.is_err());
    }

    // 16. Liquidate healthy position blocked
    #[test]
    fn test_liquidate_healthy_blocked() {
        let mut env = setup();
        let deposit_amount = 10_000_000_000u64;
        let (user, user_collateral, user_kusd) = create_user(&mut env, deposit_amount);

        deposit(&mut env, &user, &user_collateral, deposit_amount);
        mint_kusd(&mut env, &user, &user_kusd, 500_000_000); // 50% LTV, healthy

        let liquidator = Keypair::new();
        env.svm.airdrop(&liquidator.pubkey(), 10_000_000_000).unwrap();
        let liq_kusd = create_ata(&mut env.svm, &liquidator, &liquidator.pubkey(), &env.kusd_mint);
        let liq_collateral = create_ata(&mut env.svm, &liquidator, &liquidator.pubkey(), &env.collateral_mint);

        // Transfer kUSD
        let transfer_ix = spl_token::instruction::transfer(
            &spl_token::id(),
            &user_kusd,
            &liq_kusd,
            &user.pubkey(),
            &[],
            250_000_000,
        )
        .unwrap();
        let tx = Transaction::new_signed_with_payer(
            &[transfer_ix],
            Some(&user.pubkey()),
            &[&user],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        // Try to liquidate healthy position → PositionHealthy error
        let result = try_liquidate(
            &mut env,
            &liquidator,
            &user.pubkey(),
            &liq_kusd,
            &liq_collateral,
            100_000_000,
        );
        assert!(result.is_err());
    }

    // 17. Stability fee accrual over time
    #[test]
    fn test_stability_fee_accrual() {
        let mut env = setup();
        let deposit_amount = 10_000_000_000u64;
        let mint_amount = 500_000_000u64; // 500 kUSD
        let (user, user_collateral, user_kusd) = create_user(&mut env, deposit_amount);

        deposit(&mut env, &user, &user_collateral, deposit_amount);
        mint_kusd(&mut env, &user, &user_kusd, mint_amount);

        let vault_before = read_vault(&env);
        let shares_before = read_position(&env, &user.pubkey()).debt_shares;

        // Warp 1 year
        warp_clock(&mut env.svm, 365 * 24 * 3600);
        update_oracle(&mut env, SOL_PRICE); // refresh oracle timestamp

        // Accrue fees
        accrue_fees_ix(&mut env);

        let vault_after = read_vault(&env);
        assert!(vault_after.cumulative_fee_index > vault_before.cumulative_fee_index);

        // Position shares unchanged, but debt increases due to higher fee_index
        let pos = read_position(&env, &user.pubkey());
        assert_eq!(pos.debt_shares, shares_before);

        // Compute debt: shares * new_index / SCALE
        // With 2% annual fee, debt should be ~510 kUSD (500 * 1.02)
        let debt = (pos.debt_shares as u128) * vault_after.cumulative_fee_index / SCALE;
        // Allow small rounding: 509.9M to 510.1M
        assert!(debt >= 509_900_000, "Debt should be ~510M, got {}", debt);
        assert!(debt <= 510_100_000, "Debt should be ~510M, got {}", debt);
    }

    // 18. Full lifecycle
    #[test]
    fn test_full_lifecycle() {
        let mut env = setup();

        // 1. User deposits 10 SOL ($1000)
        let deposit_amount = 10_000_000_000u64;
        let (user, user_collateral, user_kusd) = create_user(&mut env, deposit_amount);
        deposit(&mut env, &user, &user_collateral, deposit_amount);

        // 2. Mint 500 kUSD (50% LTV) — well under 67% max
        let mint_amount = 500_000_000u64;
        mint_kusd(&mut env, &user, &user_kusd, mint_amount);
        assert_eq!(token_balance(&env.svm, &user_kusd), mint_amount);

        // 3. Warp 1 year, accrue fees
        warp_clock(&mut env.svm, 365 * 24 * 3600);
        update_oracle(&mut env, SOL_PRICE);
        accrue_fees_ix(&mut env);

        // 4. Verify debt > original mint (2% annual → ~510 kUSD)
        let vault = read_vault(&env);
        let pos = read_position(&env, &user.pubkey());
        let actual_debt = kusd::math::shares_to_debt(pos.debt_shares, vault.cumulative_fee_index).unwrap();
        assert!(actual_debt > mint_amount, "Debt should exceed original mint after 1yr of fees");
        assert!(actual_debt >= 509_000_000 && actual_debt <= 511_000_000,
            "Debt should be ~510M, got {}", actual_debt);

        // 5. To fully repay, we need a second user to provide kUSD for the fee gap.
        // User2 deposits collateral, mints kUSD, transfers to user1.
        let (user2, user2_collateral, user2_kusd) = create_user(&mut env, 5_000_000_000);
        deposit(&mut env, &user2, &user2_collateral, 5_000_000_000);
        let fee_gap = actual_debt - mint_amount + 1; // +1 for rounding
        mint_kusd(&mut env, &user2, &user2_kusd, fee_gap);

        // Transfer kUSD from user2 to user1
        let transfer_ix = spl_token::instruction::transfer(
            &spl_token::id(),
            &user2_kusd,
            &user_kusd,
            &user2.pubkey(),
            &[],
            fee_gap,
        )
        .unwrap();
        let tx = Transaction::new_signed_with_payer(
            &[transfer_ix],
            Some(&user2.pubkey()),
            &[&user2],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();
        env.svm.expire_blockhash();

        // 6. Repay all debt
        repay(&mut env, &user, &user.pubkey(), &user_kusd, u64::MAX);
        let pos = read_position(&env, &user.pubkey());
        assert_eq!(pos.debt_shares, 0);

        // 7. Withdraw all collateral
        withdraw(&mut env, &user, &user_collateral, deposit_amount);
        let pos = read_position(&env, &user.pubkey());
        assert_eq!(pos.collateral_amount, 0);
        assert_eq!(token_balance(&env.svm, &user_collateral), deposit_amount);
    }
}
