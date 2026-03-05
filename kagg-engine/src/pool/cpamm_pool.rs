use kagg::types::DexId;
use solana_sdk::pubkey::Pubkey;
use super::QuotablePool;

/// Off-chain representation of a constant product AMM pool (kpool/cpamm).
#[derive(Debug, Clone)]
pub struct CpammPool {
    pub address: Pubkey,
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    pub vault_a: Pubkey,
    pub vault_b: Pubkey,
    pub authority: Pubkey,
    pub reserve_a: u64,
    pub reserve_b: u64,
    pub fee_numerator: u64,   // default 30
    pub fee_denominator: u64, // default 10_000
    pub program_id: Pubkey,
}

impl QuotablePool for CpammPool {
    fn quote(&self, amount_in: u64, a_to_b: bool) -> Option<u64> {
        let (reserve_in, reserve_out) = if a_to_b {
            (self.reserve_a, self.reserve_b)
        } else {
            (self.reserve_b, self.reserve_a)
        };
        if reserve_in == 0 || reserve_out == 0 {
            return None;
        }

        let amount_with_fee =
            (amount_in as u128) * (self.fee_denominator - self.fee_numerator) as u128;
        let numerator = (reserve_out as u128) * amount_with_fee;
        let denominator =
            (reserve_in as u128) * self.fee_denominator as u128 + amount_with_fee;
        let out = (numerator / denominator) as u64;
        if out == 0 { None } else { Some(out) }
    }

    fn mint_a(&self) -> Pubkey { self.mint_a }
    fn mint_b(&self) -> Pubkey { self.mint_b }
    fn address(&self) -> Pubkey { self.address }
    fn dex_id(&self) -> DexId { DexId::Kpool }

    fn swap_accounts(&self, a_to_b: bool) -> Vec<(Pubkey, bool, bool)> {
        let input_mint = if a_to_b { self.mint_a } else { self.mint_b };
        vec![
            (self.program_id, false, false), // cpamm program
            (self.address, false, true),      // pool (mut)
            (self.authority, false, false),    // pool authority
            (self.vault_a, false, true),       // vault_a (mut)
            (self.vault_b, false, true),       // vault_b (mut)
            (input_mint, false, false),        // input mint
        ]
    }

    fn extra_data(&self, _a_to_b: bool, _amount_in: u64) -> Vec<u8> {
        vec![] // CPAMM needs no extra data
    }

    fn num_accounts(&self, _a_to_b: bool) -> u8 {
        6
    }
}
