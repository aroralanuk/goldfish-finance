use anchor_lang::prelude::*;
use anchor_spl::token::{self, MintTo, Token, Transfer};

use crate::{state::*};

use crate::{Amount, AmountKind, RoundingDirection};

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut,
              has_one = pool,
              has_one = lp_token)]
    pub pool: Account<'info, Pool>,

    /// The owner of pool
    #[account(mut)]
    pub pool: UncheckedAccount<'info>,

    /// lp tokens for pool
    #[account(mut)]
    pub lp_notes: UncheckedAccount<'info>,

    /// The address with authority to deposit the tokens
    pub approver: Signer<'info>,

    /// The source of the tokens to be deposited
    #[account(mut)]
    pub source: UncheckedAccount<'info>,

    /// The destination of the deposit notes
    #[account(mut)]
    pub destination: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
}

impl<'info> Deposit<'info> {
    fn transfer_source_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                to: self.owner.to_account_info(),
                from: self.source.to_account_info(),
                authority: self.approver.to_account_info(),
            },
        )
    }

    fn mint_lp_notes_context(&self) -> CpiContext<'_, '_, '_, 'info, MintTo<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            MintTo {
                to: self.destination.to_account_info(),
                mint: self.deposit_note_mint.to_account_info(),
                authority: self.margin_pool.to_account_info(),
            },
        )
    }
}

pub fn deposit_handler(ctx: Context<Deposit>, token_amount: u64) -> Result<()> {
    let pool = &mut ctx.accounts.pool;
    let clock = Clock::get()?;

    // Make sure interest accrual is up-to-date
    // if !pool.accrue_interest(clock.unix_timestamp) {
    //     msg!("interest accrual is too far behind");
    //     return Err(ErrorCode::InterestAccrualBehind.into());
    // }

    let rounding = RoundingDirection::direction(PoolAction::Deposit, AmountKind::Tokens);
    let lp_amount =
        pool.convert_lp_amount(Amount::tokens(token_amount), rounding)?;
    pool.deposit(&lp_amount);

    let pool = &ctx.accounts.pool;
    let signer = [&pool.signer_seeds()?[..]];

    token::transfer(
        ctx.accounts.transfer_source_context().with_signer(&signer),
        lp_amount.tokens,
    )?;
    token::mint_to(
        ctx.accounts.mint_lp_context().with_signer(&signer),
        lp_amount.notes,
    )?;

    Ok(())
}