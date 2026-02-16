#[cfg(test)]
mod make_test {
    use escrow::ESCROW_SEED;
    use escrow::ID as PROGRAM_ID;
    use mollusk_svm::{Mollusk, program::keyed_account_for_system_program, result::Check};
    use mollusk_svm_programs_token::{
        associated_token::create_account_for_associated_token_account,
        token::create_account_for_mint,
    };
    use pinocchio::{Address, error::ProgramError};
    use pinocchio_system::ID as SYSTEM_PROGRAM_ID;
    use solana_account::Account;
    use solana_instruction::{AccountMeta, Instruction};
    use solana_program_option::COption;
    use spl_token_interface::state::{Account as TokenAccount, AccountState, Mint};

    const PROGRAM_ADDRESS: Address = Address::new_from_array(PROGRAM_ID);
    const SEED: u64 = 1;

    fn setup() -> (Mollusk, Vec<(Address, Account)>, Instruction, Vec<u8>) {
        let mut mollusk = Mollusk::new_debuggable(&PROGRAM_ADDRESS, "target/deploy/escrow", true);
        mollusk_svm_programs_token::token::add_program(&mut mollusk);
        mollusk_svm_programs_token::associated_token::add_program(&mut mollusk);

        let maker_address = Address::new_unique();
        let maker_account = Account::new(100_000_000, 0, &SYSTEM_PROGRAM_ID);

        let mint_a_address = Address::new_unique();
        let mint_a_data = Mint {
            mint_authority: COption::Some(Address::new_unique()),
            supply: 10_000_000_000,
            decimals: 6,
            is_initialized: true,
            freeze_authority: COption::None,
        };
        let mint_a_account = create_account_for_mint(mint_a_data);

        let mint_b_address = Address::new_unique();
        let mint_b_data = Mint {
            mint_authority: COption::Some(Address::new_unique()),
            supply: 10_000_000_000,
            decimals: 6,
            is_initialized: true,
            freeze_authority: COption::None,
        };
        let mint_b_account = create_account_for_mint(mint_b_data);

        let (escrow_address, escrow_bump) = Address::find_program_address(
            &[
                ESCROW_SEED,
                maker_address.as_ref(),
                SEED.to_le_bytes().as_ref(),
            ],
            &PROGRAM_ADDRESS,
        );

        let maker_a_data = TokenAccount {
            mint: mint_a_address,
            owner: maker_address,
            amount: 1_000_000,
            delegate: COption::None,
            state: AccountState::Initialized,
            is_native: COption::None,
            delegated_amount: 0,
            close_authority: COption::None,
        };
        let (maker_a_ata_address, maker_a_ata_account) =
            create_account_for_associated_token_account(maker_a_data);

        let (system_program_address, system_program_account) = keyed_account_for_system_program();
        let (token_program_address, token_program_account) =
            mollusk_svm_programs_token::token::keyed_account();
        let (associated_token_program_address, associated_token_program_account) =
            mollusk_svm_programs_token::associated_token::keyed_account();

        let (calculated_maker_a_ata_address, maker_a_ata_bump) = Address::find_program_address(
            &[
                maker_address.as_ref(),
                token_program_address.as_ref(),
                mint_a_address.as_ref(),
            ],
            &associated_token_program_address,
        );

        assert!(calculated_maker_a_ata_address == maker_a_ata_address);

        let (vault_address, _) = Address::find_program_address(
            &[
                escrow_address.as_ref(),
                token_program_address.as_ref(),
                mint_a_address.as_ref(),
            ],
            &associated_token_program_address,
        );

        let accounts = vec![
            (maker_address, maker_account),
            (escrow_address, Account::default()),
            (mint_a_address, mint_a_account),
            (mint_b_address, mint_b_account),
            (maker_a_ata_address, maker_a_ata_account),
            (vault_address, Account::default()),
            (token_program_address, token_program_account),
            (
                associated_token_program_address,
                associated_token_program_account,
            ),
            (system_program_address, system_program_account),
        ];

        let amount = 1000u64;
        let receive = 1000u64;

        let discriminator = &[0u8];

        let instruction_data = [
            discriminator.as_ref(),
            &SEED.to_le_bytes(),
            &receive.to_le_bytes(),
            &amount.to_le_bytes(),
            &[maker_a_ata_bump],
            &[escrow_bump],
        ]
        .concat();

        let instruction = Instruction::new_with_bytes(
            PROGRAM_ADDRESS,
            &instruction_data,
            vec![
                AccountMeta::new(maker_address, true),
                AccountMeta::new(escrow_address, false),
                AccountMeta::new_readonly(mint_a_address, false),
                AccountMeta::new_readonly(mint_b_address, false),
                AccountMeta::new(maker_a_ata_address, false),
                AccountMeta::new(vault_address, false),
                AccountMeta::new_readonly(system_program_address, false),
                AccountMeta::new_readonly(token_program_address, false),
                AccountMeta::new_readonly(associated_token_program_address, false),
            ],
        );
        (
            mollusk,
            accounts,
            instruction,
            vec![maker_a_ata_bump, escrow_bump],
        )
    }

    #[test]
    fn make_works_correctly() {
        let (mollusk, accounts, instruction, bumps) = setup();

        let escrow_bump = bumps[1];
        let expected_escrow_data = [
            accounts[0].0.as_ref(),
            accounts[2].0.as_ref(),
            accounts[3].0.as_ref(),
            &1000u64.to_le_bytes(),
            &SEED.to_le_bytes(),
            &[escrow_bump],
            &[0u8; 7], //  padding
        ]
        .concat();

        // maker ATA: 1_000_000 - 1000 = 999_000
        let expected_maker_ata_data = create_account_for_associated_token_account(TokenAccount {
            mint: accounts[2].0,
            owner: accounts[0].0,
            amount: 999_000,
            delegate: COption::None,
            state: AccountState::Initialized,
            is_native: COption::None,
            delegated_amount: 0,
            close_authority: COption::None,
        })
        .1
        .data;

        // vault: receives 1000 tokens, owned by escrow
        let expected_vault_data = create_account_for_associated_token_account(TokenAccount {
            mint: accounts[2].0,
            owner: accounts[1].0,
            amount: 1000,
            delegate: COption::None,
            state: AccountState::Initialized,
            is_native: COption::None,
            delegated_amount: 0,
            close_authority: COption::None,
        })
        .1
        .data;

        mollusk.process_and_validate_instruction(
            &instruction,
            &accounts,
            &[
                Check::success(),
                Check::account(&accounts[1].0)
                    .data(&expected_escrow_data)
                    .build(),
                Check::account(&accounts[4].0)
                    .data(&expected_maker_ata_data)
                    .build(),
                Check::account(&accounts[5].0)
                    .data(&expected_vault_data)
                    .build(),
            ],
        );
    }

    #[test]
    fn make_fails_missing_signer() {
        let (mollusk, accounts, instruction, _) = setup();

        let mut metas = instruction.accounts.clone();
        metas[0].is_signer = false;
        let instruction = Instruction::new_with_bytes(PROGRAM_ADDRESS, &instruction.data, metas);

        mollusk.process_and_validate_instruction(
            &instruction,
            &accounts,
            &[Check::err(ProgramError::MissingRequiredSignature)],
        );
    }

    #[test]
    fn make_fails_escrow_not_empty() {
        let (mollusk, mut accounts, instruction, _) = setup();

        accounts[1].1 = Account {
            data: vec![1u8; 32],
            owner: SYSTEM_PROGRAM_ID,
            ..Account::default()
        };

        mollusk.process_and_validate_instruction(
            &instruction,
            &accounts,
            &[Check::err(ProgramError::InvalidAccountData)],
        );
    }

    #[test]
    fn make_fails_escrow_wrong_owner() {
        let (mollusk, mut accounts, instruction, _) = setup();

        accounts[1].1 = Account {
            owner: Address::new_unique(),
            ..Account::default()
        };

        mollusk.process_and_validate_instruction(
            &instruction,
            &accounts,
            &[Check::err(ProgramError::InvalidAccountOwner)],
        );
    }

    #[test]
    fn make_fails_zero_amount() {
        let (mollusk, accounts, instruction, bumps) = setup();

        let instruction_data = [
            &[0u8],
            SEED.to_le_bytes().as_ref(),
            &1000u64.to_le_bytes(),
            &0u64.to_le_bytes(),
            &[bumps[0]],
            &[bumps[1]],
        ]
        .concat();
        let instruction =
            Instruction::new_with_bytes(PROGRAM_ADDRESS, &instruction_data, instruction.accounts);

        mollusk.process_and_validate_instruction(
            &instruction,
            &accounts,
            &[Check::err(ProgramError::InvalidInstructionData)],
        );
    }

    #[test]
    fn make_fails_invalid_instruction_data() {
        let (mollusk, accounts, instruction, _) = setup();

        let instruction =
            Instruction::new_with_bytes(PROGRAM_ADDRESS, &[0u8; 5], instruction.accounts);

        mollusk.process_and_validate_instruction(
            &instruction,
            &accounts,
            &[Check::err(ProgramError::InvalidInstructionData)],
        );
    }

    #[test]
    fn make_fails_with_not_enough_accounts() {
        let (mollusk, accounts, instruction, _) = setup();

        let metas = instruction.accounts[..3].to_vec();
        let instruction = Instruction::new_with_bytes(PROGRAM_ADDRESS, &instruction.data, metas);

        mollusk.process_and_validate_instruction(
            &instruction,
            &accounts,
            &[Check::err(ProgramError::NotEnoughAccountKeys)],
        );
    }
}
