use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::state::*;

#[derive(Accounts)]
pub struct CreatePool<'info>{
     #[account(
        init,
        seeds = [token.key().as_ref()],
        bump,
        payer = payer,
        space = 8 + std::mem::size_of::<Pool>(),
    )]
    pub pool: Box<Account<'info, Pool>>,

    #[account(
        init,
        seeds = [pool.key().as_ref(), b"owner".as_ref()],
        bump,
        token::mint = token,
        token::authority = pool,
        payer = payer
    )]
    pub owner: Box<Account<'info, TokenAccount>>,

    pub token: Box<Account<'info, Mint>>,

    /// The payer of rent for new accounts
    #[account(mut)]
    pub payer: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn create_pool_handler(ctx: Context<CreatePool>) -> Result<()> {
    let pool = &mut ctx.accounts.pool;

    pool.address = pool.key();
    pool.pool_bump[0] = *ctx.bumps.get("pool").unwrap();
    pool.token = ctx.accounts.token.key();
    pool.owner = ctx.accounts.owner.key();

    let clock = Clock::get()?;
    pool.accrued_until = clock.unix_timestamp;

    Ok(())
}