#![allow(unused)]

use anchor_lang::prelude::*;

declare_id!("6GrfrMykKD4mcFgVzdnW2ySEgpqiYVEDHxms9Kj2ubfo");

mod error;
mod instructions;
mod states;
mod utils;

use instructions::*;

#[program]
pub mod escrow {
    use super::*;

    // Admin Calls
    pub fn init_main_state(
        ctx: Context<AInitMainState>,
        fee_receiver: Pubkey,
        fee_rate: u64,
        holding_time: i64,
    ) -> Result<()> {
        instructions::init_main_state(ctx, fee_receiver, fee_rate, holding_time)?;
        Ok(())
    }

    pub fn update_main_state(
        ctx: Context<AUpdateMainState>,
        fee_receiver: Pubkey,
        fee_rate: u64,
        holding_time: i64,
    ) -> Result<()> {
        instructions::update_main_state(ctx, fee_receiver, fee_rate, holding_time)?;
        Ok(())
    }

    pub fn update_main_state_owner(
        ctx: Context<AUpdateMainState>,
        new_owner: Pubkey,
    ) -> Result<()> {
        let state = &mut ctx.accounts.main_state;
        state.owner = new_owner;
        Ok(())
    }

    // UserSide calls
    pub fn create_vault(ctx: Context<ACreateVault>, input: CreateVaultInput) -> Result<()> {
        instructions::create_vault(ctx, input)?;
        Ok(())
    }

    pub fn revert_payment(ctx: Context<ARevertPayment>) -> Result<()> {
        instructions::revert_payment(ctx)?;
        Ok(())
    }

    pub fn redeem_payment(ctx: Context<ARedeedPayment>) -> Result<()> {
        instructions::redeem_payment(ctx)?;
        Ok(())
    }
}
