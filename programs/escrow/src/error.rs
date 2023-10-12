use anchor_lang::prelude::error_code;

#[error_code]
pub enum EscrowError {
    #[msg("This methods only call by specfic owner only")]
    UnAuthorisedCaller,

    #[msg("Tokens are released for reciver")]
    TokensAreReleased,

    #[msg("Tokens are released not")]
    TokensAreNotReleased,
}
