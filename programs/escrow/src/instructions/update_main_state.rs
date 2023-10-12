use crate::{error::EscrowError, states::MainState, utils::SEED_MAIN_STATE};
use anchor_lang::prelude::*;

pub(crate) fn update_main_state(
    ctx: Context<AUpdateMainState>,
    fee_receiver: Pubkey,
    fee_rate: u64,
    holding_time: i64,
) -> Result<()> {
    let state = &mut ctx.accounts.main_state;
    state.fee_receiver = fee_receiver;
    state.fee_rate = fee_rate;
    state.holding_time = holding_time;

    Ok(())
}

#[derive(Accounts)]
pub struct AUpdateMainState<'info> {
    #[account(mut, address = main_state.owner @ EscrowError::UnAuthorisedCaller)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds =[SEED_MAIN_STATE],
        bump,
    )]
    pub main_state: Account<'info, MainState>,
}
