use anchor_lang::prelude::*;
use pyth_sdk_solana::PriceFeed;
use jet_proto_math::Number;

use crate::{Amount, AmountKind, RoundingDirection, ErrorCode};


#[account]
pub struct Pool {
    pub pool_bump: [u8;1],
    pub address: Pubkey,
    pub owner: Pubkey,
    pub token: Pubkey,
    pub lp_notes_mint: Pubkey,
    pub loan_note_mint: Pubkey,
    pub lp_notes: u64,
    pub token_price: u64,
    pub deposited_tokens: u64,
    pub borrowed_tokens: [u8; 24],
    pub uncollected_fees: [u8; 24],
    pub accrued_until: i64,
}

pub struct BorrowerAccount {
    pub bump_seed: [u8; 1],
    pub user_seed: [u8; 2],

    /// The owner of this account, which generally has to sign for any changes to it
    pub owner: Pubkey,

    /// The state of an active liquidation for this account
    pub creditRating: u64,

    /// The storage for tracking account balances
    pub loans: [u8; 24],
}

impl Pool {
    pub fn signer_seeds(&self) -> Result<[&[u8]; 2]> {
        // TODO: error checking

        Ok([self.token.as_ref(), self.pool_bump.as_ref()])
    }

     fn convert_amount(
        &self,
        amount: Amount,
        exchange_rate: Number,
        rounding: RoundingDirection,
    ) -> Result<FullAmount> {
        let amount = match amount.kind {
            AmountKind::Tokens => FullAmount {
                tokens: amount.value,
                notes: match rounding {
                    RoundingDirection::Down => {
                         (Number::from(amount.value) / exchange_rate).as_u64(0)
                    }
                    RoundingDirection::Up => {
                        (Number::from(amount.value) / exchange_rate).as_u64_ceil(0)
                    }
                },
            },

            AmountKind::Notes => FullAmount {
                notes: amount.value,
                tokens: match rounding {
                    RoundingDirection::Down => {
                        (Number::from(amount.value) * exchange_rate).as_u64(0)
                    }
                    RoundingDirection::Up => {
                        (Number::from(amount.value) * exchange_rate).as_u64_ceil(0)
                    }
                },
            },
        };

        // As FullAmount represents the conversion of tokens to/from notes for
        // the purpose of:
        // - adding/subtracting tokens to/from a pool's vault
        // - minting/burning notes from a pool's deposit/loan mint.
        if (amount.notes == 0 && amount.tokens > 0) || (amount.tokens == 0 && amount.notes > 0) {
            return err!(crate::ErrorCode::InvalidAmount);
        }
        Ok(amount)
    }

    /// Convert the amount to be representable by tokens and notes for deposits
    pub fn convert_lp_note_amount(
        &self,
        amount: Amount,
        rounding: RoundingDirection,
    ) -> Result<FullAmount> {
        self.convert_amount(amount, self.notes_exchange_rate(), rounding)
    }

    /// Get the exchange rate for deposit note -> token
    fn notes_exchange_rate(&self) -> Number {
        let deposit_notes = std::cmp::max(1, self.lp_notes);
        let total_value = std::cmp::max(Number::ONE, self.total_value());
        (total_value - *self.total_uncollected_fees()) / Number::from(deposit_notes)
    }


    /// Gets the total value of assets owned by/owed to the pool.
    fn total_value(&self) -> Number {
        *self.total_borrowed() + Number::from(self.deposited_tokens)
    }

    fn total_uncollected_fees(&self) -> &Number {
        bytemuck::from_bytes(&self.uncollected_fees)
    }

    fn total_borrowed(&self) -> &Number {
        bytemuck::from_bytes(&self.borrowed_tokens)
    }
}

#[derive(Debug)]
pub struct FullAmount {
    pub tokens: u64,
    pub notes: u64,
}