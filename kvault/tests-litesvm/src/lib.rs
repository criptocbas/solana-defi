#[cfg(test)]
mod tests {
    use anchor_lang::{system_program, InstructionData, ToAccountMetas};
    use anchor_spl::associated_token::get_associated_token_address;
    use litesvm::LiteSVM;
    use solana_sdk::{
        clock::Clock,
        instruction::{AccountMeta, Instruction},
        program_pack::Pack,
        pubkey::Pubkey,
        signature::{Keypair, Signer},
        transaction::Transaction,
    };
    use std::str::FromStr;

    use klend::constants::*;
    use klend::state::ReserveConfig;
    use kvault::constants::*;

    const KLEND_PROGRAM_ID: &str = "D91U4ZA4bcSWRNhqAf9oMPBMNYhEkwZNooXPUUZSM68v";
    const KVAULT_PROGRAM_ID: &str = "FEiBosN66wZt8wYTzpUPoCeqqzbKG9FATmeWnm8RNZE1";

    fn klend_id() -> Pubkey {
        Pubkey::from_str(KLEND_PROGRAM_ID).unwrap()
    }

    fn kvault_id() -> Pubkey {
        Pubkey::from_str(KVAULT_PROGRAM_ID).unwrap()
    }

    // ── klend PDA helpers ──────────────────────────────────────

    fn lending_market_pda(admin: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[LENDING_MARKET_SEED, admin.as_ref()], &klend_id())
    }

    fn reserve_pda(lending_market: &Pubkey, token_mint: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[RESERVE_SEED, lending_market.as_ref(), token_mint.as_ref()],
            &klend_id(),
        )
    }

    fn reserve_authority_pda(reserve: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[RESERVE_AUTHORITY_SEED, reserve.as_ref()], &klend_id())
    }

    fn obligation_pda(lending_market: &Pubkey, owner: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[OBLIGATION_SEED, lending_market.as_ref(), owner.as_ref()],
            &klend_id(),
        )
    }

    fn mock_oracle_pda(token_mint: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[MOCK_ORACLE_SEED, token_mint.as_ref()], &klend_id())
    }

    // ── kvault PDA helpers ─────────────────────────────────────

    fn vault_pda(underlying_mint: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[VAULT_SEED, underlying_mint.as_ref()], &kvault_id())
    }

    fn vault_authority_pda(vault: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[VAULT_AUTHORITY_SEED, vault.as_ref()], &kvault_id())
    }

    fn share_mint_pda(vault: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[SHARE_MINT_SEED, vault.as_ref()], &kvault_id())
    }

    // ── SPL helpers ────────────────────────────────────────────

    fn create_mint(
        svm: &mut LiteSVM,
        payer: &Keypair,
        authority: &Pubkey,
        decimals: u8,
    ) -> Pubkey {
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
        let mint_data = spl_token::state::Mint::unpack(&data.data).unwrap();
        mint_data.supply
    }

    fn warp_clock(svm: &mut LiteSVM, seconds_forward: i64) {
        let mut clock: Clock = svm.get_sysvar();
        clock.unix_timestamp += seconds_forward;
        svm.set_sysvar(&clock);
    }

    // ── klend configs ──────────────────────────────────────────

    const RATE_4_PCT: u64 = 40_000_000_000_000_000;
    const RATE_300_PCT: u64 = 3_000_000_000_000_000_000;
    const UTIL_80_PCT: u64 = 800_000_000_000_000_000;

    fn sol_config() -> ReserveConfig {
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

    const SOL_PRICE: u64 = 100_000_000; // $100 * 1e6
    const USDC_PRICE: u64 = 1_000_000;  // $1 * 1e6
    const SOL_DECIMALS: u8 = 9;
    const USDC_DECIMALS: u8 = 6;

    // ── klend instruction builders ─────────────────────────────

    fn klend_init_market_ix(admin: &Pubkey) -> Instruction {
        let (lending_market, _) = lending_market_pda(admin);
        Instruction {
            program_id: klend_id(),
            accounts: klend::accounts::InitMarket {
                admin: *admin,
                lending_market,
                system_program: system_program::ID,
            }
            .to_account_metas(None),
            data: klend::instruction::InitMarket {}.data(),
        }
    }

    fn klend_init_mock_oracle_ix(
        payer: &Pubkey,
        token_mint: &Pubkey,
        price: u64,
        decimals: u8,
    ) -> Instruction {
        let (oracle, _) = mock_oracle_pda(token_mint);
        Instruction {
            program_id: klend_id(),
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

    fn klend_update_mock_oracle_ix(payer: &Pubkey, token_mint: &Pubkey, price: u64) -> Instruction {
        let (oracle, _) = mock_oracle_pda(token_mint);
        Instruction {
            program_id: klend_id(),
            accounts: klend::accounts::UpdateMockOracle {
                payer: *payer,
                oracle,
            }
            .to_account_metas(None),
            data: klend::instruction::UpdateMockOracle { price }.data(),
        }
    }

    fn klend_init_reserve_ix(
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
            program_id: klend_id(),
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

    fn klend_refresh_reserve_ix(reserve: &Pubkey, oracle: &Pubkey) -> Instruction {
        Instruction {
            program_id: klend_id(),
            accounts: klend::accounts::RefreshReserve {
                reserve: *reserve,
                oracle: *oracle,
            }
            .to_account_metas(None),
            data: klend::instruction::RefreshReserve {}.data(),
        }
    }

    fn klend_deposit_ix(
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
            program_id: klend_id(),
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

    fn klend_borrow_ix(
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
        extra_accounts: Vec<AccountMeta>,
    ) -> Instruction {
        let (obligation, _) = obligation_pda(lending_market, user);
        let user_token_account = get_associated_token_address(user, borrow_mint);
        let mut accounts = klend::accounts::Borrow {
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
        .to_account_metas(None);
        accounts.extend(extra_accounts);
        Instruction {
            program_id: klend_id(),
            accounts,
            data: klend::instruction::Borrow { amount }.data(),
        }
    }

    /// Build remaining_accounts for klend health check: [(reserve, oracle), ...]
    fn position_accounts(pairs: &[(&Pubkey, &Pubkey)]) -> Vec<AccountMeta> {
        pairs
            .iter()
            .flat_map(|(reserve, oracle)| {
                vec![
                    AccountMeta::new_readonly(**reserve, false),
                    AccountMeta::new_readonly(**oracle, false),
                ]
            })
            .collect()
    }

    // ── kvault instruction builders ────────────────────────────

    fn kvault_init_vault_ix(
        admin: &Pubkey,
        underlying_mint: &Pubkey,
        klend_reserve: &Pubkey,
        performance_fee_bps: u16,
        management_fee_bps: u16,
        deposit_cap: u64,
    ) -> Instruction {
        let (vault, _) = vault_pda(underlying_mint);
        let (vault_auth, _) = vault_authority_pda(&vault);
        let (share_mint, _) = share_mint_pda(&vault);
        let vault_token_account = get_associated_token_address(&vault_auth, underlying_mint);
        Instruction {
            program_id: kvault_id(),
            accounts: kvault::accounts::InitVault {
                admin: *admin,
                underlying_mint: *underlying_mint,
                vault,
                vault_authority: vault_auth,
                share_mint,
                vault_token_account,
                klend_reserve: *klend_reserve,
                klend_program: klend_id(),
                token_program: spl_token::id(),
                associated_token_program: spl_associated_token_account::id(),
                system_program: system_program::ID,
            }
            .to_account_metas(None),
            data: kvault::instruction::InitVault {
                performance_fee_bps,
                management_fee_bps,
                deposit_cap,
            }
            .data(),
        }
    }

    fn kvault_deposit_ix(
        user: &Pubkey,
        underlying_mint: &Pubkey,
        amount: u64,
    ) -> Instruction {
        let (vault, _) = vault_pda(underlying_mint);
        let (vault_auth, _) = vault_authority_pda(&vault);
        let (share_mint, _) = share_mint_pda(&vault);
        let vault_token_account = get_associated_token_address(&vault_auth, underlying_mint);
        let user_token_account = get_associated_token_address(user, underlying_mint);
        let user_share_account = get_associated_token_address(user, &share_mint);
        Instruction {
            program_id: kvault_id(),
            accounts: kvault::accounts::Deposit {
                user: *user,
                vault,
                vault_authority: vault_auth,
                share_mint,
                vault_token_account,
                user_token_account,
                user_share_account,
                token_program: spl_token::id(),
            }
            .to_account_metas(None),
            data: kvault::instruction::Deposit { amount }.data(),
        }
    }

    fn kvault_withdraw_ix(
        user: &Pubkey,
        underlying_mint: &Pubkey,
        shares: u64,
    ) -> Instruction {
        let (vault, _) = vault_pda(underlying_mint);
        let (vault_auth, _) = vault_authority_pda(&vault);
        let (share_mint, _) = share_mint_pda(&vault);
        let vault_token_account = get_associated_token_address(&vault_auth, underlying_mint);
        let user_token_account = get_associated_token_address(user, underlying_mint);
        let user_share_account = get_associated_token_address(user, &share_mint);
        Instruction {
            program_id: kvault_id(),
            accounts: kvault::accounts::Withdraw {
                user: *user,
                vault,
                vault_authority: vault_auth,
                share_mint,
                vault_token_account,
                user_token_account,
                user_share_account,
                token_program: spl_token::id(),
            }
            .to_account_metas(None),
            data: kvault::instruction::Withdraw { shares }.data(),
        }
    }

    fn kvault_allocate_ix(
        admin: &Pubkey,
        underlying_mint: &Pubkey,
        lending_market: &Pubkey,
        klend_reserve: &Pubkey,
        klend_token_vault: &Pubkey,
        amount: u64,
    ) -> Instruction {
        let (vault, _) = vault_pda(underlying_mint);
        let (vault_auth, _) = vault_authority_pda(&vault);
        let vault_token_account = get_associated_token_address(&vault_auth, underlying_mint);
        let (klend_obligation, _) = obligation_pda(lending_market, &vault_auth);
        Instruction {
            program_id: kvault_id(),
            accounts: kvault::accounts::Allocate {
                admin: *admin,
                vault,
                vault_authority: vault_auth,
                vault_token_account,
                lending_market: *lending_market,
                klend_reserve: *klend_reserve,
                klend_obligation,
                klend_token_vault: *klend_token_vault,
                klend_program: klend_id(),
                token_program: spl_token::id(),
                system_program: system_program::ID,
            }
            .to_account_metas(None),
            data: kvault::instruction::Allocate { amount }.data(),
        }
    }

    fn kvault_deallocate_ix(
        admin: &Pubkey,
        underlying_mint: &Pubkey,
        lending_market: &Pubkey,
        klend_reserve: &Pubkey,
        klend_reserve_authority: &Pubkey,
        klend_token_vault: &Pubkey,
        klend_oracle: &Pubkey,
        klend_shares: u64,
    ) -> Instruction {
        let (vault, _) = vault_pda(underlying_mint);
        let (vault_auth, _) = vault_authority_pda(&vault);
        let vault_token_account = get_associated_token_address(&vault_auth, underlying_mint);
        let (klend_obligation, _) = obligation_pda(lending_market, &vault_auth);
        Instruction {
            program_id: kvault_id(),
            accounts: kvault::accounts::Deallocate {
                admin: *admin,
                vault,
                vault_authority: vault_auth,
                vault_token_account,
                lending_market: *lending_market,
                klend_reserve: *klend_reserve,
                klend_reserve_authority: *klend_reserve_authority,
                klend_obligation,
                klend_token_vault: *klend_token_vault,
                klend_oracle: *klend_oracle,
                klend_program: klend_id(),
                token_program: spl_token::id(),
            }
            .to_account_metas(None),
            data: kvault::instruction::Deallocate { klend_shares }.data(),
        }
    }

    fn kvault_harvest_ix(
        admin: &Pubkey,
        underlying_mint: &Pubkey,
        lending_market: &Pubkey,
        klend_reserve: &Pubkey,
        fee_recipient: &Pubkey,
    ) -> Instruction {
        let (vault, _) = vault_pda(underlying_mint);
        let (vault_auth, _) = vault_authority_pda(&vault);
        let (share_mint, _) = share_mint_pda(&vault);
        let vault_token_account = get_associated_token_address(&vault_auth, underlying_mint);
        let (klend_obligation, _) = obligation_pda(lending_market, &vault_auth);
        let fee_recipient_share_account = get_associated_token_address(fee_recipient, &share_mint);
        Instruction {
            program_id: kvault_id(),
            accounts: kvault::accounts::Harvest {
                admin: *admin,
                vault,
                vault_authority: vault_auth,
                share_mint,
                vault_token_account,
                klend_reserve: *klend_reserve,
                klend_obligation,
                fee_recipient_share_account,
                token_program: spl_token::id(),
            }
            .to_account_metas(None),
            data: kvault::instruction::Harvest {}.data(),
        }
    }

    fn kvault_set_halt_ix(
        admin: &Pubkey,
        underlying_mint: &Pubkey,
        halted: bool,
    ) -> Instruction {
        let (vault, _) = vault_pda(underlying_mint);
        Instruction {
            program_id: kvault_id(),
            accounts: kvault::accounts::SetHalt {
                vault,
                admin: *admin,
            }
            .to_account_metas(None),
            data: kvault::instruction::SetHalt { halted }.data(),
        }
    }

    // ── Test environment ───────────────────────────────────────

    struct TestEnv {
        svm: LiteSVM,
        admin: Keypair,
        mint_authority: Keypair,
        // klend
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
        // kvault
        vault: Pubkey,
        vault_authority: Pubkey,
        share_mint: Pubkey,
        vault_token_account: Pubkey,
    }

    fn setup() -> TestEnv {
        let mut svm = LiteSVM::new();
        let admin = Keypair::new();
        svm.airdrop(&admin.pubkey(), 100_000_000_000).unwrap();

        // Load both programs
        let klend_bytes = include_bytes!("../../../klend/target/deploy/klend.so");
        svm.add_program(klend_id(), klend_bytes).unwrap();
        let kvault_bytes = include_bytes!("../../target/deploy/kvault.so");
        svm.add_program(kvault_id(), kvault_bytes).unwrap();

        // Create mint authority
        let mint_authority = Keypair::new();
        svm.airdrop(&mint_authority.pubkey(), 1_000_000_000)
            .unwrap();

        // Create SOL and USDC mints
        let sol_mint = create_mint(&mut svm, &admin, &mint_authority.pubkey(), SOL_DECIMALS);
        let usdc_mint = create_mint(&mut svm, &admin, &mint_authority.pubkey(), USDC_DECIMALS);

        // Init klend lending market
        let ix = klend_init_market_ix(&admin.pubkey());
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&admin.pubkey()),
            &[&admin],
            svm.latest_blockhash(),
        );
        svm.send_transaction(tx).unwrap();
        let (lending_market, _) = lending_market_pda(&admin.pubkey());

        // Init klend oracles
        svm.expire_blockhash();
        let ix1 = klend_init_mock_oracle_ix(&admin.pubkey(), &sol_mint, SOL_PRICE, SOL_DECIMALS);
        let ix2 = klend_init_mock_oracle_ix(&admin.pubkey(), &usdc_mint, USDC_PRICE, USDC_DECIMALS);
        let tx = Transaction::new_signed_with_payer(
            &[ix1, ix2],
            Some(&admin.pubkey()),
            &[&admin],
            svm.latest_blockhash(),
        );
        svm.send_transaction(tx).unwrap();

        let (sol_oracle, _) = mock_oracle_pda(&sol_mint);
        let (usdc_oracle, _) = mock_oracle_pda(&usdc_mint);

        // Init klend reserves
        svm.expire_blockhash();
        let ix = klend_init_reserve_ix(&admin.pubkey(), &lending_market, &sol_mint, sol_config());
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&admin.pubkey()),
            &[&admin],
            svm.latest_blockhash(),
        );
        svm.send_transaction(tx).unwrap();

        svm.expire_blockhash();
        let ix =
            klend_init_reserve_ix(&admin.pubkey(), &lending_market, &usdc_mint, usdc_config());
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

        // Init kvault for USDC
        svm.expire_blockhash();
        let ix = kvault_init_vault_ix(
            &admin.pubkey(),
            &usdc_mint,
            &usdc_reserve,
            1000, // 10% performance fee
            200,  // 2% management fee
            0,    // no cap
        );
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&admin.pubkey()),
            &[&admin],
            svm.latest_blockhash(),
        );
        svm.send_transaction(tx).unwrap();

        let (vault, _) = vault_pda(&usdc_mint);
        let (vault_authority, _) = vault_authority_pda(&vault);
        let (share_mint, _) = share_mint_pda(&vault);
        let vault_token_account = get_associated_token_address(&vault_authority, &usdc_mint);

        // Create admin's share ATA for fee receipt
        svm.expire_blockhash();
        create_ata(&mut svm, &admin, &admin.pubkey(), &share_mint);

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
            vault,
            vault_authority,
            share_mint,
            vault_token_account,
        }
    }

    /// Update oracle timestamps (needed after warp_clock to prevent staleness)
    fn update_oracles(env: &mut TestEnv) {
        env.svm.expire_blockhash();
        let ix1 = klend_update_mock_oracle_ix(&env.admin.pubkey(), &env.sol_mint, SOL_PRICE);
        let ix2 = klend_update_mock_oracle_ix(&env.admin.pubkey(), &env.usdc_mint, USDC_PRICE);
        let tx = Transaction::new_signed_with_payer(
            &[ix1, ix2],
            Some(&env.admin.pubkey()),
            &[&env.admin],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();
    }

    fn refresh_reserves(env: &mut TestEnv) {
        env.svm.expire_blockhash();
        let ix1 = klend_refresh_reserve_ix(&env.sol_reserve, &env.sol_oracle);
        let ix2 = klend_refresh_reserve_ix(&env.usdc_reserve, &env.usdc_oracle);
        let tx = Transaction::new_signed_with_payer(
            &[ix1, ix2],
            Some(&env.admin.pubkey()),
            &[&env.admin],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();
    }

    /// Fund a user with USDC tokens and create share ATA
    fn fund_user_usdc(env: &mut TestEnv, user: &Keypair, usdc_amount: u64) {
        env.svm
            .airdrop(&user.pubkey(), 10_000_000_000)
            .unwrap();
        let ata = create_ata(&mut env.svm, &env.admin, &user.pubkey(), &env.usdc_mint);
        mint_tokens(
            &mut env.svm,
            &env.admin,
            &env.usdc_mint,
            &ata,
            &env.mint_authority,
            usdc_amount,
        );
        // Create share ATA
        env.svm.expire_blockhash();
        create_ata(&mut env.svm, &env.admin, &user.pubkey(), &env.share_mint);
    }

    fn klend_repay_ix(
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
            program_id: klend_id(),
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

    /// Set up a borrower: deposits SOL as collateral, borrows USDC to generate yield.
    /// Returns the borrower Keypair so they can repay later.
    fn setup_borrower(env: &mut TestEnv, sol_deposit: u64, usdc_borrow: u64) -> Keypair {
        let borrower = Keypair::new();
        env.svm
            .airdrop(&borrower.pubkey(), 10_000_000_000)
            .unwrap();

        // Fund borrower with SOL tokens
        let sol_ata = create_ata(&mut env.svm, &env.admin, &borrower.pubkey(), &env.sol_mint);
        mint_tokens(
            &mut env.svm,
            &env.admin,
            &env.sol_mint,
            &sol_ata,
            &env.mint_authority,
            sol_deposit,
        );

        // Create USDC ATA for borrower
        env.svm.expire_blockhash();
        create_ata(
            &mut env.svm,
            &env.admin,
            &borrower.pubkey(),
            &env.usdc_mint,
        );

        // Refresh reserves
        refresh_reserves(env);

        // Deposit SOL as collateral
        env.svm.expire_blockhash();
        let ix = klend_deposit_ix(
            &borrower.pubkey(),
            &env.lending_market,
            &env.sol_reserve,
            &env.sol_mint,
            &env.sol_vault,
            sol_deposit,
        );
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&borrower.pubkey()),
            &[&borrower],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        // Refresh reserves before borrow
        refresh_reserves(env);

        // Borrow USDC
        env.svm.expire_blockhash();
        let ix = klend_borrow_ix(
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
            position_accounts(&[(&env.sol_reserve, &env.sol_oracle), (&env.usdc_reserve, &env.usdc_oracle)]),
        );
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&borrower.pubkey()),
            &[&borrower],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        borrower
    }

    /// Have borrower repay their USDC loan (need to mint extra USDC for interest)
    fn repay_borrower(env: &mut TestEnv, borrower: &Keypair, repay_amount: u64) {
        // Mint extra USDC to cover interest
        let borrower_usdc = get_associated_token_address(&borrower.pubkey(), &env.usdc_mint);
        let current = token_balance(&env.svm, &borrower_usdc);
        if current < repay_amount {
            env.svm.expire_blockhash();
            mint_tokens(
                &mut env.svm,
                &env.admin,
                &env.usdc_mint,
                &borrower_usdc,
                &env.mint_authority,
                repay_amount - current,
            );
        }

        refresh_reserves(env);
        env.svm.expire_blockhash();
        let refresh_ix = klend_refresh_reserve_ix(&env.usdc_reserve, &env.usdc_oracle);
        let repay_ix = klend_repay_ix(
            &borrower.pubkey(),
            &borrower.pubkey(),
            &env.lending_market,
            &env.usdc_reserve,
            &env.usdc_mint,
            &env.usdc_vault,
            repay_amount,
        );
        let tx = Transaction::new_signed_with_payer(
            &[refresh_ix, repay_ix],
            Some(&borrower.pubkey()),
            &[borrower],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();
    }

    // ========== TESTS ==========

    #[test]
    fn test_01_init_vault() {
        let env = setup();

        // Verify vault account exists
        let account = env.svm.get_account(&env.vault).unwrap();
        assert!(account.data.len() > 8);

        // Verify share mint exists
        let supply = mint_supply(&env.svm, &env.share_mint);
        assert_eq!(supply, 0);

        // Verify vault token account exists with 0 balance
        assert_eq!(token_balance(&env.svm, &env.vault_token_account), 0);

        // Verify vault authority is funded
        let auth_account = env.svm.get_account(&env.vault_authority).unwrap();
        assert!(auth_account.lamports >= 100_000_000);

        println!("Vault initialized at {}", env.vault);
        println!("Share mint: {}", env.share_mint);
        println!("Vault authority: {}", env.vault_authority);
    }

    #[test]
    fn test_02_deposit() {
        let mut env = setup();
        let user = Keypair::new();
        let deposit_amount = 1_000_000u64; // 1 USDC
        fund_user_usdc(&mut env, &user, deposit_amount);

        env.svm.expire_blockhash();
        let ix = kvault_deposit_ix(&user.pubkey(), &env.usdc_mint, deposit_amount);
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&user.pubkey()),
            &[&user],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        // Verify vault received tokens
        assert_eq!(token_balance(&env.svm, &env.vault_token_account), deposit_amount);

        // Verify user received shares
        let user_share_ata = get_associated_token_address(&user.pubkey(), &env.share_mint);
        let shares = token_balance(&env.svm, &user_share_ata);
        assert!(shares > 0);
        println!("Deposited {} USDC, received {} shares", deposit_amount, shares);
    }

    #[test]
    fn test_03_second_deposit_proportional() {
        let mut env = setup();
        let user1 = Keypair::new();
        let user2 = Keypair::new();
        let deposit_amount = 1_000_000u64; // 1 USDC each
        fund_user_usdc(&mut env, &user1, deposit_amount);
        fund_user_usdc(&mut env, &user2, deposit_amount);

        // User1 deposits
        env.svm.expire_blockhash();
        let ix = kvault_deposit_ix(&user1.pubkey(), &env.usdc_mint, deposit_amount);
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&user1.pubkey()),
            &[&user1],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        // User2 deposits same amount
        env.svm.expire_blockhash();
        let ix = kvault_deposit_ix(&user2.pubkey(), &env.usdc_mint, deposit_amount);
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&user2.pubkey()),
            &[&user2],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        let user1_shares = token_balance(
            &env.svm,
            &get_associated_token_address(&user1.pubkey(), &env.share_mint),
        );
        let user2_shares = token_balance(
            &env.svm,
            &get_associated_token_address(&user2.pubkey(), &env.share_mint),
        );

        // Both should get approximately equal shares (within rounding)
        let diff = if user1_shares > user2_shares {
            user1_shares - user2_shares
        } else {
            user2_shares - user1_shares
        };
        assert!(diff <= 1, "Share difference too large: {}", diff);
        println!(
            "User1: {} shares, User2: {} shares",
            user1_shares, user2_shares
        );
    }

    #[test]
    fn test_04_allocate_to_klend() {
        let mut env = setup();
        let user = Keypair::new();
        let deposit_amount = 100_000_000u64; // 100 USDC
        fund_user_usdc(&mut env, &user, deposit_amount);

        // Deposit to vault
        env.svm.expire_blockhash();
        let ix = kvault_deposit_ix(&user.pubkey(), &env.usdc_mint, deposit_amount);
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&user.pubkey()),
            &[&user],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        let idle_before = token_balance(&env.svm, &env.vault_token_account);
        assert_eq!(idle_before, deposit_amount);

        // Refresh klend reserve before allocate
        refresh_reserves(&mut env);

        // Allocate 50 USDC to klend
        let allocate_amount = 50_000_000u64;
        env.svm.expire_blockhash();
        let refresh_ix = klend_refresh_reserve_ix(&env.usdc_reserve, &env.usdc_oracle);
        let alloc_ix = kvault_allocate_ix(
            &env.admin.pubkey(),
            &env.usdc_mint,
            &env.lending_market,
            &env.usdc_reserve,
            &env.usdc_vault,
            allocate_amount,
        );
        let tx = Transaction::new_signed_with_payer(
            &[refresh_ix, alloc_ix],
            Some(&env.admin.pubkey()),
            &[&env.admin],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        // Verify idle decreased
        let idle_after = token_balance(&env.svm, &env.vault_token_account);
        assert_eq!(idle_after, deposit_amount - allocate_amount);

        // Verify klend vault received tokens
        let klend_vault_balance = token_balance(&env.svm, &env.usdc_vault);
        assert_eq!(klend_vault_balance, allocate_amount);

        println!(
            "Allocated {} USDC to klend. Idle: {} -> {}",
            allocate_amount, idle_before, idle_after
        );
    }

    #[test]
    fn test_05_harvest_yield() {
        let mut env = setup();
        let user = Keypair::new();
        let deposit_amount = 100_000_000u64; // 100 USDC
        fund_user_usdc(&mut env, &user, deposit_amount);

        // Deposit to vault
        env.svm.expire_blockhash();
        let ix = kvault_deposit_ix(&user.pubkey(), &env.usdc_mint, deposit_amount);
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&user.pubkey()),
            &[&user],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        // Allocate all to klend
        refresh_reserves(&mut env);
        env.svm.expire_blockhash();
        let refresh_ix = klend_refresh_reserve_ix(&env.usdc_reserve, &env.usdc_oracle);
        let alloc_ix = kvault_allocate_ix(
            &env.admin.pubkey(),
            &env.usdc_mint,
            &env.lending_market,
            &env.usdc_reserve,
            &env.usdc_vault,
            deposit_amount,
        );
        let tx = Transaction::new_signed_with_payer(
            &[refresh_ix, alloc_ix],
            Some(&env.admin.pubkey()),
            &[&env.admin],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        // Setup borrower to generate yield
        setup_borrower(
            &mut env,
            10_000_000_000, // 10 SOL collateral ($1000)
            50_000_000,     // borrow 50 USDC
        );

        let supply_before = mint_supply(&env.svm, &env.share_mint);

        // Warp time 1 year for interest to accrue
        warp_clock(&mut env.svm, 365 * 24 * 3600);
        update_oracles(&mut env);

        // Refresh reserves (accrues interest)
        refresh_reserves(&mut env);

        // Harvest
        env.svm.expire_blockhash();
        let ix = kvault_harvest_ix(
            &env.admin.pubkey(),
            &env.usdc_mint,
            &env.lending_market,
            &env.usdc_reserve,
            &env.admin.pubkey(),
        );
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&env.admin.pubkey()),
            &[&env.admin],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        let supply_after = mint_supply(&env.svm, &env.share_mint);
        let fee_shares_minted = supply_after - supply_before;

        println!(
            "Harvest: supply {} -> {} (fee shares: {})",
            supply_before, supply_after, fee_shares_minted
        );
        assert!(
            fee_shares_minted > 0,
            "Expected fee shares to be minted from yield"
        );
    }

    #[test]
    fn test_06_exchange_rate_increases() {
        let mut env = setup();
        let user = Keypair::new();
        let deposit_amount = 100_000_000u64; // 100 USDC
        fund_user_usdc(&mut env, &user, deposit_amount);

        // Deposit
        env.svm.expire_blockhash();
        let ix = kvault_deposit_ix(&user.pubkey(), &env.usdc_mint, deposit_amount);
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&user.pubkey()),
            &[&user],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        let user_shares = token_balance(
            &env.svm,
            &get_associated_token_address(&user.pubkey(), &env.share_mint),
        );

        // Allocate to klend
        refresh_reserves(&mut env);
        env.svm.expire_blockhash();
        let refresh_ix = klend_refresh_reserve_ix(&env.usdc_reserve, &env.usdc_oracle);
        let alloc_ix = kvault_allocate_ix(
            &env.admin.pubkey(),
            &env.usdc_mint,
            &env.lending_market,
            &env.usdc_reserve,
            &env.usdc_vault,
            deposit_amount,
        );
        let tx = Transaction::new_signed_with_payer(
            &[refresh_ix, alloc_ix],
            Some(&env.admin.pubkey()),
            &[&env.admin],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        // Borrower generates yield (80 USDC at 80% utilization -> 4% annual rate)
        let borrower = setup_borrower(&mut env, 10_000_000_000, 80_000_000);

        // Warp 1 year
        warp_clock(&mut env.svm, 365 * 24 * 3600);
        update_oracles(&mut env);
        refresh_reserves(&mut env);

        // Harvest
        env.svm.expire_blockhash();
        let ix = kvault_harvest_ix(
            &env.admin.pubkey(),
            &env.usdc_mint,
            &env.lending_market,
            &env.usdc_reserve,
            &env.admin.pubkey(),
        );
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&env.admin.pubkey()),
            &[&env.admin],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        // Borrower repays loan so klend has full liquidity for deallocate
        repay_borrower(&mut env, &borrower, 90_000_000);

        // Re-harvest to update total_invested to current klend share value (post-repay)
        env.svm.expire_blockhash();
        let ix = kvault_harvest_ix(
            &env.admin.pubkey(),
            &env.usdc_mint,
            &env.lending_market,
            &env.usdc_reserve,
            &env.admin.pubkey(),
        );
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&env.admin.pubkey()),
            &[&env.admin],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        // Deallocate everything to make funds available for withdrawal
        let (klend_obligation_key, _) =
            obligation_pda(&env.lending_market, &env.vault_authority);
        let obligation_data = env.svm.get_account(&klend_obligation_key).unwrap();
        let obligation: klend::state::Obligation =
            anchor_lang::AccountDeserialize::try_deserialize(&mut &obligation_data.data[..])
                .unwrap();
        let vault_klend_shares = obligation
            .deposits
            .iter()
            .find(|d| d.reserve == env.usdc_reserve)
            .map(|d| d.shares)
            .unwrap_or(0);

        refresh_reserves(&mut env);
        env.svm.expire_blockhash();
        let refresh_ix = klend_refresh_reserve_ix(&env.usdc_reserve, &env.usdc_oracle);
        let dealloc_ix = kvault_deallocate_ix(
            &env.admin.pubkey(),
            &env.usdc_mint,
            &env.lending_market,
            &env.usdc_reserve,
            &env.usdc_reserve_authority,
            &env.usdc_vault,
            &env.usdc_oracle,
            vault_klend_shares,
        );
        let tx = Transaction::new_signed_with_payer(
            &[refresh_ix, dealloc_ix],
            Some(&env.admin.pubkey()),
            &[&env.admin],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        // Now withdraw user's shares
        env.svm.expire_blockhash();
        let ix = kvault_withdraw_ix(&user.pubkey(), &env.usdc_mint, user_shares);
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&user.pubkey()),
            &[&user],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        let user_usdc = token_balance(
            &env.svm,
            &get_associated_token_address(&user.pubkey(), &env.usdc_mint),
        );
        println!(
            "Deposited {} USDC, withdrew {} USDC (profit: {})",
            deposit_amount,
            user_usdc,
            user_usdc.saturating_sub(deposit_amount)
        );
        assert!(
            user_usdc > deposit_amount,
            "Expected user to withdraw more than deposited"
        );
    }

    #[test]
    fn test_07_withdraw() {
        let mut env = setup();
        let user = Keypair::new();
        let deposit_amount = 10_000_000u64; // 10 USDC
        fund_user_usdc(&mut env, &user, deposit_amount);

        // Deposit
        env.svm.expire_blockhash();
        let ix = kvault_deposit_ix(&user.pubkey(), &env.usdc_mint, deposit_amount);
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&user.pubkey()),
            &[&user],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        let user_shares = token_balance(
            &env.svm,
            &get_associated_token_address(&user.pubkey(), &env.share_mint),
        );

        // Withdraw all shares
        env.svm.expire_blockhash();
        let ix = kvault_withdraw_ix(&user.pubkey(), &env.usdc_mint, user_shares);
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&user.pubkey()),
            &[&user],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        let user_usdc = token_balance(
            &env.svm,
            &get_associated_token_address(&user.pubkey(), &env.usdc_mint),
        );
        // Should get back the deposit (minus rounding)
        assert!(
            user_usdc >= deposit_amount - 1,
            "Expected to recover deposit, got {}",
            user_usdc
        );
        println!("Withdrew {} USDC from {} deposited", user_usdc, deposit_amount);
    }

    #[test]
    fn test_08_deallocate() {
        let mut env = setup();
        let user = Keypair::new();
        let deposit_amount = 100_000_000u64; // 100 USDC
        fund_user_usdc(&mut env, &user, deposit_amount);

        // Deposit
        env.svm.expire_blockhash();
        let ix = kvault_deposit_ix(&user.pubkey(), &env.usdc_mint, deposit_amount);
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&user.pubkey()),
            &[&user],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        // Allocate to klend
        refresh_reserves(&mut env);
        env.svm.expire_blockhash();
        let refresh_ix = klend_refresh_reserve_ix(&env.usdc_reserve, &env.usdc_oracle);
        let alloc_ix = kvault_allocate_ix(
            &env.admin.pubkey(),
            &env.usdc_mint,
            &env.lending_market,
            &env.usdc_reserve,
            &env.usdc_vault,
            deposit_amount,
        );
        let tx = Transaction::new_signed_with_payer(
            &[refresh_ix, alloc_ix],
            Some(&env.admin.pubkey()),
            &[&env.admin],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        let idle_after_alloc = token_balance(&env.svm, &env.vault_token_account);
        assert_eq!(idle_after_alloc, 0);

        // Get klend shares from obligation
        let (klend_obligation_key, _) =
            obligation_pda(&env.lending_market, &env.vault_authority);
        let obligation_data = env.svm.get_account(&klend_obligation_key).unwrap();
        let obligation: klend::state::Obligation =
            anchor_lang::AccountDeserialize::try_deserialize(&mut &obligation_data.data[..])
                .unwrap();
        let vault_klend_shares = obligation
            .deposits
            .iter()
            .find(|d| d.reserve == env.usdc_reserve)
            .map(|d| d.shares)
            .unwrap_or(0);

        // Deallocate all
        refresh_reserves(&mut env);
        env.svm.expire_blockhash();
        let refresh_ix = klend_refresh_reserve_ix(&env.usdc_reserve, &env.usdc_oracle);
        let dealloc_ix = kvault_deallocate_ix(
            &env.admin.pubkey(),
            &env.usdc_mint,
            &env.lending_market,
            &env.usdc_reserve,
            &env.usdc_reserve_authority,
            &env.usdc_vault,
            &env.usdc_oracle,
            vault_klend_shares,
        );
        let tx = Transaction::new_signed_with_payer(
            &[refresh_ix, dealloc_ix],
            Some(&env.admin.pubkey()),
            &[&env.admin],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        let idle_after_dealloc = token_balance(&env.svm, &env.vault_token_account);
        // Should have recovered approximately the full amount (minus rounding)
        assert!(
            idle_after_dealloc >= deposit_amount - 1,
            "Expected ~{} idle, got {}",
            deposit_amount,
            idle_after_dealloc
        );
        println!(
            "Deallocated: idle {} -> {}",
            idle_after_alloc, idle_after_dealloc
        );
    }

    #[test]
    fn test_09_emergency_halt() {
        let mut env = setup();
        let user = Keypair::new();
        fund_user_usdc(&mut env, &user, 10_000_000);

        // Halt vault
        env.svm.expire_blockhash();
        let ix = kvault_set_halt_ix(&env.admin.pubkey(), &env.usdc_mint, true);
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&env.admin.pubkey()),
            &[&env.admin],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        // Try deposit -- should fail
        env.svm.expire_blockhash();
        let ix = kvault_deposit_ix(&user.pubkey(), &env.usdc_mint, 1_000_000);
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&user.pubkey()),
            &[&user],
            env.svm.latest_blockhash(),
        );
        let result = env.svm.send_transaction(tx);
        assert!(result.is_err(), "Deposit should fail when halted");
        println!("Deposit correctly blocked when halted");
    }

    #[test]
    fn test_10_withdraw_during_halt() {
        let mut env = setup();
        let user = Keypair::new();
        let deposit_amount = 5_000_000u64;
        fund_user_usdc(&mut env, &user, deposit_amount);

        // Deposit first
        env.svm.expire_blockhash();
        let ix = kvault_deposit_ix(&user.pubkey(), &env.usdc_mint, deposit_amount);
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&user.pubkey()),
            &[&user],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        let user_shares = token_balance(
            &env.svm,
            &get_associated_token_address(&user.pubkey(), &env.share_mint),
        );

        // Halt vault
        env.svm.expire_blockhash();
        let ix = kvault_set_halt_ix(&env.admin.pubkey(), &env.usdc_mint, true);
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&env.admin.pubkey()),
            &[&env.admin],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        // Withdraw should succeed even when halted
        env.svm.expire_blockhash();
        let ix = kvault_withdraw_ix(&user.pubkey(), &env.usdc_mint, user_shares);
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&user.pubkey()),
            &[&user],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        let user_usdc = token_balance(
            &env.svm,
            &get_associated_token_address(&user.pubkey(), &env.usdc_mint),
        );
        assert!(
            user_usdc >= deposit_amount - 1,
            "User should be able to withdraw during halt"
        );
        println!("Withdraw during halt succeeded: {} USDC", user_usdc);
    }

    #[test]
    fn test_11_deposit_cap() {
        let mut env = setup();

        // Re-init vault with a 50 USDC cap (we need a separate mint for a new vault,
        // or just test by modifying vault state... instead let's use a user deposit
        // that exceeds the cap)
        // Since we initialized with cap=0 (no cap), let's test by creating a new setup
        // with a capped vault. But we can't easily do that with our current setup fn.
        // Instead, let's verify the cap logic by just checking the code path.
        // We'll create a fresh env with a cap.

        // Create a new test from scratch with deposit cap
        let mut svm = LiteSVM::new();
        let admin = Keypair::new();
        svm.airdrop(&admin.pubkey(), 100_000_000_000).unwrap();

        let klend_bytes = include_bytes!("../../../klend/target/deploy/klend.so");
        svm.add_program(klend_id(), klend_bytes).unwrap();
        let kvault_bytes = include_bytes!("../../target/deploy/kvault.so");
        svm.add_program(kvault_id(), kvault_bytes).unwrap();

        let mint_authority = Keypair::new();
        svm.airdrop(&mint_authority.pubkey(), 1_000_000_000)
            .unwrap();

        let usdc_mint = create_mint(&mut svm, &admin, &mint_authority.pubkey(), USDC_DECIMALS);

        // Init klend (needed for vault init -- it stores klend_reserve)
        let ix = klend_init_market_ix(&admin.pubkey());
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&admin.pubkey()),
            &[&admin],
            svm.latest_blockhash(),
        );
        svm.send_transaction(tx).unwrap();
        let (lending_market, _) = lending_market_pda(&admin.pubkey());

        svm.expire_blockhash();
        let ix = klend_init_mock_oracle_ix(&admin.pubkey(), &usdc_mint, USDC_PRICE, USDC_DECIMALS);
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&admin.pubkey()),
            &[&admin],
            svm.latest_blockhash(),
        );
        svm.send_transaction(tx).unwrap();

        svm.expire_blockhash();
        let ix = klend_init_reserve_ix(&admin.pubkey(), &lending_market, &usdc_mint, usdc_config());
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&admin.pubkey()),
            &[&admin],
            svm.latest_blockhash(),
        );
        svm.send_transaction(tx).unwrap();
        let (usdc_reserve, _) = reserve_pda(&lending_market, &usdc_mint);

        // Init vault with 10 USDC cap
        let cap = 10_000_000u64; // 10 USDC
        svm.expire_blockhash();
        let ix = kvault_init_vault_ix(
            &admin.pubkey(),
            &usdc_mint,
            &usdc_reserve,
            1000,
            200,
            cap,
        );
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&admin.pubkey()),
            &[&admin],
            svm.latest_blockhash(),
        );
        svm.send_transaction(tx).unwrap();

        let (vault, _) = vault_pda(&usdc_mint);
        let (_, _) = vault_authority_pda(&vault);
        let (share_mint, _) = share_mint_pda(&vault);

        // Fund user with 20 USDC
        let user = Keypair::new();
        svm.airdrop(&user.pubkey(), 10_000_000_000).unwrap();
        let user_ata = create_ata(&mut svm, &admin, &user.pubkey(), &usdc_mint);
        mint_tokens(&mut svm, &admin, &usdc_mint, &user_ata, &mint_authority, 20_000_000);
        svm.expire_blockhash();
        create_ata(&mut svm, &admin, &user.pubkey(), &share_mint);

        // Deposit 5 USDC -- should succeed
        svm.expire_blockhash();
        let ix = kvault_deposit_ix(&user.pubkey(), &usdc_mint, 5_000_000);
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&user.pubkey()),
            &[&user],
            svm.latest_blockhash(),
        );
        svm.send_transaction(tx).unwrap();
        println!("5 USDC deposit succeeded under 10 USDC cap");

        // Deposit 6 USDC -- should fail (5 + 6 = 11 > 10)
        svm.expire_blockhash();
        let ix = kvault_deposit_ix(&user.pubkey(), &usdc_mint, 6_000_000);
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&user.pubkey()),
            &[&user],
            svm.latest_blockhash(),
        );
        let result = svm.send_transaction(tx);
        assert!(result.is_err(), "Deposit should fail when exceeding cap");
        println!("6 USDC deposit correctly rejected (would exceed cap)");
    }

    #[test]
    fn test_12_full_lifecycle() {
        let mut env = setup();
        let user = Keypair::new();
        let deposit_amount = 100_000_000u64; // 100 USDC
        fund_user_usdc(&mut env, &user, deposit_amount);

        // 1. Deposit
        env.svm.expire_blockhash();
        let ix = kvault_deposit_ix(&user.pubkey(), &env.usdc_mint, deposit_amount);
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&user.pubkey()),
            &[&user],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();
        println!("1. Deposited {} USDC", deposit_amount);

        let user_shares = token_balance(
            &env.svm,
            &get_associated_token_address(&user.pubkey(), &env.share_mint),
        );

        // 2. Allocate to klend
        refresh_reserves(&mut env);
        env.svm.expire_blockhash();
        let refresh_ix = klend_refresh_reserve_ix(&env.usdc_reserve, &env.usdc_oracle);
        let alloc_ix = kvault_allocate_ix(
            &env.admin.pubkey(),
            &env.usdc_mint,
            &env.lending_market,
            &env.usdc_reserve,
            &env.usdc_vault,
            deposit_amount,
        );
        let tx = Transaction::new_signed_with_payer(
            &[refresh_ix, alloc_ix],
            Some(&env.admin.pubkey()),
            &[&env.admin],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();
        println!("2. Allocated {} USDC to klend", deposit_amount);

        // 3. Borrower generates yield (80 USDC at 80% utilization -> 4% annual rate)
        let borrower = setup_borrower(&mut env, 10_000_000_000, 80_000_000);
        println!("3. Borrower took 80 USDC loan");

        // 4. Warp 6 months
        warp_clock(&mut env.svm, 182 * 24 * 3600);
        update_oracles(&mut env);
        refresh_reserves(&mut env);
        println!("4. Warped 6 months forward");

        // 5. Harvest
        env.svm.expire_blockhash();
        let ix = kvault_harvest_ix(
            &env.admin.pubkey(),
            &env.usdc_mint,
            &env.lending_market,
            &env.usdc_reserve,
            &env.admin.pubkey(),
        );
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&env.admin.pubkey()),
            &[&env.admin],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();
        println!("5. Harvested yield");

        let total_supply = mint_supply(&env.svm, &env.share_mint);
        let admin_fee_shares = token_balance(
            &env.svm,
            &get_associated_token_address(&env.admin.pubkey(), &env.share_mint),
        );
        println!(
            "   Total supply: {}, User shares: {}, Fee shares: {}",
            total_supply, user_shares, admin_fee_shares
        );

        // 6. Borrower repays so klend has full liquidity
        repay_borrower(&mut env, &borrower, 90_000_000);
        println!("6. Borrower repaid loan");

        // Re-harvest to sync total_invested with current klend share value (post-repay)
        env.svm.expire_blockhash();
        let ix = kvault_harvest_ix(
            &env.admin.pubkey(),
            &env.usdc_mint,
            &env.lending_market,
            &env.usdc_reserve,
            &env.admin.pubkey(),
        );
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&env.admin.pubkey()),
            &[&env.admin],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        // 7. Partial withdraw (half shares)
        let half_shares = user_shares / 2;

        // Need to deallocate first to have idle funds
        let (klend_obligation_key, _) =
            obligation_pda(&env.lending_market, &env.vault_authority);
        let obligation_data = env.svm.get_account(&klend_obligation_key).unwrap();
        let obligation: klend::state::Obligation =
            anchor_lang::AccountDeserialize::try_deserialize(&mut &obligation_data.data[..])
                .unwrap();
        let vault_klend_shares = obligation
            .deposits
            .iter()
            .find(|d| d.reserve == env.usdc_reserve)
            .map(|d| d.shares)
            .unwrap_or(0);

        // Deallocate half of klend shares
        let dealloc_shares = vault_klend_shares / 2;
        refresh_reserves(&mut env);
        env.svm.expire_blockhash();
        let refresh_ix = klend_refresh_reserve_ix(&env.usdc_reserve, &env.usdc_oracle);
        let dealloc_ix = kvault_deallocate_ix(
            &env.admin.pubkey(),
            &env.usdc_mint,
            &env.lending_market,
            &env.usdc_reserve,
            &env.usdc_reserve_authority,
            &env.usdc_vault,
            &env.usdc_oracle,
            dealloc_shares,
        );
        let tx = Transaction::new_signed_with_payer(
            &[refresh_ix, dealloc_ix],
            Some(&env.admin.pubkey()),
            &[&env.admin],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();
        println!("7. Deallocated half from klend");

        env.svm.expire_blockhash();
        let ix = kvault_withdraw_ix(&user.pubkey(), &env.usdc_mint, half_shares);
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&user.pubkey()),
            &[&user],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();
        let user_usdc_after_partial = token_balance(
            &env.svm,
            &get_associated_token_address(&user.pubkey(), &env.usdc_mint),
        );
        println!(
            "   Partial withdraw: {} shares -> {} USDC",
            half_shares, user_usdc_after_partial
        );

        // 8. Deallocate remaining and full withdraw
        let obligation_data = env.svm.get_account(&klend_obligation_key).unwrap();
        let obligation: klend::state::Obligation =
            anchor_lang::AccountDeserialize::try_deserialize(&mut &obligation_data.data[..])
                .unwrap();
        let remaining_klend_shares = obligation
            .deposits
            .iter()
            .find(|d| d.reserve == env.usdc_reserve)
            .map(|d| d.shares)
            .unwrap_or(0);

        if remaining_klend_shares > 0 {
            refresh_reserves(&mut env);
            env.svm.expire_blockhash();
            let refresh_ix = klend_refresh_reserve_ix(&env.usdc_reserve, &env.usdc_oracle);
            let dealloc_ix = kvault_deallocate_ix(
                &env.admin.pubkey(),
                &env.usdc_mint,
                &env.lending_market,
                &env.usdc_reserve,
                &env.usdc_reserve_authority,
                &env.usdc_vault,
                &env.usdc_oracle,
                remaining_klend_shares,
            );
            let tx = Transaction::new_signed_with_payer(
                &[refresh_ix, dealloc_ix],
                Some(&env.admin.pubkey()),
                &[&env.admin],
                env.svm.latest_blockhash(),
            );
            env.svm.send_transaction(tx).unwrap();
        }

        let remaining_user_shares = token_balance(
            &env.svm,
            &get_associated_token_address(&user.pubkey(), &env.share_mint),
        );

        env.svm.expire_blockhash();
        let ix = kvault_withdraw_ix(&user.pubkey(), &env.usdc_mint, remaining_user_shares);
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&user.pubkey()),
            &[&user],
            env.svm.latest_blockhash(),
        );
        env.svm.send_transaction(tx).unwrap();

        let final_user_usdc = token_balance(
            &env.svm,
            &get_associated_token_address(&user.pubkey(), &env.usdc_mint),
        );
        let final_user_shares = token_balance(
            &env.svm,
            &get_associated_token_address(&user.pubkey(), &env.share_mint),
        );

        println!("8. Full lifecycle complete:");
        println!("   User deposited: {} USDC", deposit_amount);
        println!("   User recovered: {} USDC", final_user_usdc);
        println!("   User shares remaining: {}", final_user_shares);
        println!(
            "   Net profit: {} USDC",
            final_user_usdc.saturating_sub(deposit_amount)
        );

        assert_eq!(final_user_shares, 0, "User should have 0 shares remaining");
        assert!(
            final_user_usdc > deposit_amount,
            "User should have profited from yield"
        );
    }
}
