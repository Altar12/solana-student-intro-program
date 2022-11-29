pub mod instruction;
pub mod state;

use borsh::BorshSerialize;
use instruction::StudentIntroInstruction;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    borsh::try_from_slice_unchecked,
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    pubkey::Pubkey,
    system_instruction,
    sysvar::{rent::Rent, Sysvar},
};
use state::StudentIntroAccountState;
use std::convert::TryInto;

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = StudentIntroInstruction::unpack(instruction_data)?;
    match instruction {
        StudentIntroInstruction::AddStudentIntro { name, msg } => {
            add_student_intro(program_id, accounts, name, msg)
        }
    }
}

pub fn add_student_intro(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    name: String,
    msg: String,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let initializer = next_account_info(account_info_iter)?;
    let pda_account = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;

    let (pda, bump) = Pubkey::find_program_address(
        &[initializer.key.as_ref(), name.as_bytes().as_ref()],
        program_id,
    );
    msg!("Found PDA: {}", pda);
    let data_len = 1 + 4 + name.len() + 4 + msg.len();
    let rent_amt = Rent::get()?.minimum_balance(data_len);

    invoke_signed(
        &system_instruction::create_account(
            &initializer.key,
            &pda_account.key,
            rent_amt,
            data_len.try_into().unwrap(),
            program_id,
        ),
        &[
            initializer.clone(),
            pda_account.clone(),
            system_program.clone(),
        ],
        &[&[initializer.key.as_ref(), name.as_bytes().as_ref(), &[bump]]],
    )?;
    msg!("Created PDA account successfully");
    msg!("Deserializing account data");
    msg!("Name: {}", name.clone());
    msg!("Msg: {}", msg.clone());
    let mut account_data =
        try_from_slice_unchecked::<StudentIntroAccountState>(&pda_account.data.borrow()).unwrap();
    account_data.name = name;
    account_data.msg = msg;
    account_data.is_initialized = true;
    msg!("Serializing account data");
    account_data.serialize(&mut &mut pda_account.data.borrow_mut()[..])?;
    msg!("Serialization successful");
    Ok(())
}
