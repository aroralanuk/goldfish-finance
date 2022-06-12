// SPDX-License-Identifier: AGPL-3.0-or-later

use anchor_lang::prelude::*;
use anchor_spl::token::{self, MintTo, Token, TokenAccount};

use crate::{state::*, AmountKind};
use crate::{Amount, ErrorCode};

#[derive(Accounts)]
pub struct RequestBorrow<'info> {
    /// The margin account being executed on
    #[account(signer)]
    pub borrower_account: AccountLoader<'info, BorrowerAccount>,

    /// The pool to borrow from
    #[account(mut)]
    pub pool: Account<'info, Pool>,

    pub token_program: Program<'info, Token>,
}

impl<'info> RequestBorrow<'info> {
    fn mint_loan_context(&self) -> CpiContext<'_, '_, '_, 'info, MintTo<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            MintTo {
                mint: self.loan_note_mint.to_account_info(),
                to: self.loan_account.to_account_info(),
                authority: self.margin_pool.to_account_info(),
            },
        )
    }

    fn mint_deposit_context(&self) -> CpiContext<'_, '_, '_, 'info, MintTo<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            MintTo {
                to: self.deposit_account.to_account_info(),
                mint: self.deposit_note_mint.to_account_info(),
                authority: self.margin_pool.to_account_info(),
            },
        )
    }
}

pub fn request_borrow_handler(ctx: Context<MarginBorrow>, token_amount: u64) -> Result<()> {
    let pool = &mut ctx.accounts.margin_pool;
    let clock = Clock::get()?;

    // // Make sure interest accrual is up-to-date
    // if !pool.accrue_interest(clock.unix_timestamp) {
    //     msg!("interest accrual is too far behind");
    //     return Err(ErrorCode::InterestAccrualBehind.into());
    // }

    // First record a borrow of the tokens requested
    let borrow_rounding = RoundingDirection::direction(PoolAction::Borrow, AmountKind::Tokens);
    let borrow_amount = pool.convert_loan_amount(Amount::tokens(token_amount), borrow_rounding)?;
    pool.borrow(&borrow_amount)?;

    

    // Then record a deposit of the same borrowed tokens
    let deposit_rounding = RoundingDirection::direction(PoolAction::Deposit, AmountKind::Tokens);
    let deposit_amount =
        pool.convert_deposit_amount(Amount::tokens(token_amount), deposit_rounding)?;
    pool.deposit(&deposit_amount);

    // Finish by minting the loan and deposit notes
    let pool = &ctx.accounts.margin_pool;
    let signer = [&pool.signer_seeds()?[..]];

    Ok(())
}
