use anchor_lang::prelude::*;

#[account]
pub struct MainState {
    pub owner: Pubkey,
    pub fee_receiver: Pubkey,
    pub fee_rate: u64,
    pub holding_time: i64,
}
impl MainState {
    pub const MAX_SIZE: usize = std::mem::size_of::<Self>();
}

#[account]
pub struct VaultState {
    pub sender: Pubkey,
    pub receiver: Pubkey,
    pub id: u64,
    pub release_time: i64,
    pub token: Pubkey,
}
impl VaultState {
    pub const MAX_SIZE: usize = std::mem::size_of::<Self>();
}
