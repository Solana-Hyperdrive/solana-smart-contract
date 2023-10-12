use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::{
    error::EscrowError,
    states::{MainState, VaultState},
    utils::{transfer_tokens_from_vault, SEED_MAIN_STATE, SEED_VAULT_STATE},
};

pub(crate) fn revert_payment(ctx: Context<ARevertPayment>) -> Result<()> {
    let vault = &mut ctx.accounts.vault_state;
    let vault_ata = ctx.accounts.vault_ata.to_account_info();
    let sender_ata = ctx.accounts.sender_ata.to_account_info();
    let token_program = ctx.accounts.token_program.to_account_info();
    let amount = ctx.accounts.vault_ata.amount;

    transfer_tokens_from_vault(
        vault,
        vault_ata,
        sender_ata,
        token_program,
        amount,
        ctx.bumps["vault_state"],
    )?;

    vault.close(ctx.accounts.sender.to_account_info())?;
    Ok(())
}

#[derive(Accounts)]
pub struct ARevertPayment<'info> {
    #[account(mut, address = vault_state.sender @ EscrowError::UnAuthorisedCaller)]
    pub sender: Signer<'info>,

    #[account(
        mut,
        token::mint = vault_state.token,
        token::authority = sender,
    )]
    pub sender_ata: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [SEED_VAULT_STATE, sender.key().as_ref(), vault_state.id.to_le_bytes().as_ref()],
        bump,
        constraint = Clock::get()?.unix_timestamp <= vault_state.release_time @ EscrowError::TokensAreReleased,
    )]
    pub vault_state: Box<Account<'info, VaultState>>,

    #[account(
        mut,
        token::mint = vault_state.token,
        token::authority = vault_state
    )]
    pub vault_ata: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
