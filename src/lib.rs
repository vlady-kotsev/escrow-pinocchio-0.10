use pinocchio::{AccountView, Address, ProgramResult, entrypoint, error::ProgramError};
mod constants;
mod instructions;
mod state;
pub use constants::*;
use instructions::*;
use pinocchio_pubkey::declare_id;
pub use state::*;

declare_id!("4AtAjt1xrf6SwnwSh8GjnTJFrkMVZBscPSKKaFG8mQam");
entrypoint!(process_instruction);

pub fn process_instruction(
    _program_id: &Address,
    accounts: &[AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    let [disc, data @ ..] = instruction_data else {
        return Err(ProgramError::InvalidInstructionData);
    };
    match disc {
        Make::DISCRIMINATOR => Make::try_from((accounts, data))?.process(),
        Take::DISCRIMINATOR => Take::try_from((accounts, data))?.process(),
        _ => Err(ProgramError::InvalidInstructionData),
    }
}
