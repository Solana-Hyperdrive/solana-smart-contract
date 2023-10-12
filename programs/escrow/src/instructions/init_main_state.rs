use crate::{states::MainState, utils::SEED_MAIN_STATE};
use anchor_lang::prelude::*;

pub(crate) fn init_main_state(
    ctx: Context<AInitMainState>,
    fee_receiver: Pubkey,
    fee_rate: u64,
    holding_time: i64,
) -> Result<()> {
    let state = &mut ctx.accounts.main_state;
    state.owner = ctx.accounts.owner.key();
    state.fee_receiver = fee_receiver;
    state.fee_rate = fee_rate;
    state.holding_time = holding_time;
    Ok(())
}

#[derive(Accounts)]
pub struct AInitMainState<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        init,
        payer = owner,
        seeds =[SEED_MAIN_STATE],
        bump,
        space = 8 + MainState::MAX_SIZE
    )]
    pub main_state: Account<'info, MainState>,

    pub system_program: Program<'info, System>,
}
