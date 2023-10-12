use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};

use crate::{
    states::{MainState, VaultState},
    utils::{transfer_tokens, MAX_FEE_REATE, SEED_MAIN_STATE, SEED_VAULT_STATE},
};

#[derive(Default, Copy, AnchorSerialize, AnchorDeserialize, Clone)]
pub struct CreateVaultInput {
    pub id: u64,
    pub receiver: Pubkey,
    pub amount: u64,
}

pub(crate) fn create_vault(ctx: Context<ACreateVault>, input: CreateVaultInput) -> Result<()> {
    let main_state = &mut ctx.accounts.main_state;
    let sender = ctx.accounts.sender.to_account_info();
    let sender_ata = ctx.accounts.sender_ata.to_account_info();
    let vault = &mut ctx.accounts.vault;
    let vault_ata = ctx.accounts.vault_ata.to_account_info();
    let fee_receiver_ata = ctx.accounts.fee_receiver_ata.to_account_info();
    let token_program = ctx.accounts.token_program.to_account_info();

    // init state
    vault.id = input.id;
    vault.sender = sender.key();
    vault.receiver = input.receiver;
    vault.token = ctx.accounts.token.key();
    vault.release_time = Clock::get()?.unix_timestamp + main_state.holding_time;

    msg!("free rate: {}", main_state.fee_rate);

    // transfer fees
    let fee = ((input.amount as u128)
        .checked_mul(main_state.fee_rate as u128)
        .unwrap()
        / MAX_FEE_REATE) as u64;

    transfer_tokens(
        sender.to_account_info(),
        sender_ata.to_account_info(),
        fee_receiver_ata,
        token_program.to_account_info(),
        fee,
    )?;

    // transfer token to the vault
    transfer_tokens(
        sender.to_account_info(),
        sender_ata.to_account_info(),
        vault_ata,
        token_program.to_account_info(),
        input.amount,
    )?;

    Ok(())
}

#[derive(Accounts)]
#[instruction(input: CreateVaultInput)]
pub struct ACreateVault<'info> {
    #[account(mut)]
    pub sender: Signer<'info>,

    #[account(mut)]
    pub token: Box<Account<'info, Mint>>,

    #[account(
        mut,
        token::mint = token,
        token::authority = sender
    )]
    pub sender_ata: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        seeds =[SEED_MAIN_STATE],
        bump,
    )]
    pub main_state: Account<'info, MainState>,

    #[account(
        init,
        payer = sender,
        seeds = [SEED_VAULT_STATE, sender.key().as_ref(), input.id.to_le_bytes().as_ref()],
        bump,
        space = 8 + VaultState::MAX_SIZE,
    )]
    pub vault: Box<Account<'info, VaultState>>,

    #[account(
        init,
        payer = sender,
        associated_token::mint = token,
        associated_token::authority = vault
    )]
    pub vault_ata: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        token::mint = token,
        token::authority = main_state.fee_receiver
    )]
    pub fee_receiver_ata: Box<Account<'info, TokenAccount>>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,

    pub system_program: Program<'info, System>,
}
