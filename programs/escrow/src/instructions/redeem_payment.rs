use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::{
    error::EscrowError,
    states::{MainState, VaultState},
    utils::{transfer_tokens_from_vault, SEED_MAIN_STATE, SEED_VAULT_STATE},
};

pub(crate) fn redeem_payment(ctx: Context<ARedeedPayment>) -> Result<()> {
    let vault = &mut ctx.accounts.vault_state;
    let vault_ata = ctx.accounts.vault_ata.to_account_info();
    let receiver_ata = ctx.accounts.receiver_ata.to_account_info();
    let token_program = ctx.accounts.token_program.to_account_info();
    let amount = ctx.accounts.vault_ata.amount;

    transfer_tokens_from_vault(
        vault,
        vault_ata,
        receiver_ata,
        token_program,
        amount,
        ctx.bumps["vault_state"],
    )?;

    vault.close(ctx.accounts.sender.to_account_info())?;
    Ok(())
}

#[derive(Accounts)]
pub struct ARedeedPayment<'info> {
    #[account(mut, address = vault_state.receiver @ EscrowError::UnAuthorisedCaller)]
    pub receiver: Signer<'info>,

    #[account(
        mut,
        token::mint= vault_state.token,
        token::authority = receiver
    )]
    pub receiver_ata: Box<Account<'info, TokenAccount>>,

    ///CHECK: to give back rent amount
    #[account(mut, address = vault_state.sender)]
    pub sender: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [SEED_VAULT_STATE, sender.key().as_ref(), vault_state.id.to_le_bytes().as_ref()],
        bump,
        constraint = Clock::get()?.unix_timestamp >= vault_state.release_time @ EscrowError::TokensAreNotReleased,
    )]
    pub vault_state: Box<Account<'info, VaultState>>,

    #[account(
        mut,
        token::mint= vault_state.token,
        token::authority = vault_state
    )]
    pub vault_ata: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
