use anchor_lang::solana_program::program_option::COption;
use anchor_lang::solana_program::program_pack::Pack;
use anchor_lang::{AccountDeserialize, InstructionData, ToAccountMetas};
use anchor_spl::token::spl_token::state::{Account as SplAccount, AccountState, Mint};
use anchor_spl::token::ID as TOKEN_PROGRAM_ID;
use solana_program_test::{processor, ProgramTest, ProgramTestContext};
use solana_sdk::account::Account;
use solana_sdk::instruction::Instruction;
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::transaction::Transaction;

// Anchor's generated entrypoint ties all three outer references to one lifetime,
// while ProgramTest's native ABI exposes separate lifetimes. This adapter keeps
// the temporary reborrow local to the synchronous entrypoint call.
fn solgov_processor<'program, 'accounts, 'info, 'data>(
    program_id: &'program solana_sdk::pubkey::Pubkey,
    accounts: &'accounts [anchor_lang::solana_program::account_info::AccountInfo<'info>],
    instruction_data: &'data [u8],
) -> anchor_lang::solana_program::entrypoint::ProgramResult {
    // SAFETY: Anchor's entrypoint does not retain any reference beyond this call.
    let (program_id, accounts, instruction_data) = unsafe {
        (
            program_id,
            std::mem::transmute::<
                &'accounts [anchor_lang::solana_program::account_info::AccountInfo<'info>],
                &'program [anchor_lang::solana_program::account_info::AccountInfo<'program>],
            >(accounts),
            std::mem::transmute::<&'data [u8], &'program [u8]>(instruction_data),
        )
    };
    solgov::entry(program_id, accounts, instruction_data)
}

fn transfer_instruction(
    from: &solana_sdk::pubkey::Pubkey,
    to: &solana_sdk::pubkey::Pubkey,
    lamports: u64,
) -> Instruction {
    let mut data = 2_u32.to_le_bytes().to_vec();
    data.extend_from_slice(&lamports.to_le_bytes());
    Instruction {
        program_id: anchor_lang::system_program::ID,
        accounts: vec![
            solana_sdk::instruction::AccountMeta::new(*from, true),
            solana_sdk::instruction::AccountMeta::new(*to, false),
        ],
        data,
    }
}

async fn send_instruction(
    context: &mut ProgramTestContext,
    instruction: Instruction,
    additional_signers: &[&Keypair],
) -> Result<(), Box<dyn std::error::Error>> {
    let blockhash = context.banks_client.get_latest_blockhash().await?;
    let mut signers = vec![&context.payer];
    signers.extend_from_slice(additional_signers);
    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&context.payer.pubkey()),
        &signers,
        blockhash,
    );
    context
        .banks_client
        .process_transaction(transaction)
        .await?;
    Ok(())
}

#[tokio::test]
async fn proposal_vote_and_treasury_execution_follow_the_onchain_rules(
) -> Result<(), Box<dyn std::error::Error>> {
    let governance_mint = Keypair::new().pubkey();
    let voter = Keypair::new();
    let voter_token_account = Keypair::new().pubkey();

    let mint_state = Mint {
        mint_authority: COption::Some(voter.pubkey()),
        supply: 10,
        decimals: 0,
        is_initialized: true,
        freeze_authority: COption::None,
    };
    let mut mint_data = vec![0; Mint::LEN];
    Mint::pack(mint_state, &mut mint_data)?;
    let token_state = SplAccount {
        mint: governance_mint,
        owner: voter.pubkey(),
        amount: 10,
        delegate: COption::None,
        state: AccountState::Initialized,
        is_native: COption::None,
        delegated_amount: 0,
        close_authority: COption::None,
    };
    let mut token_data = vec![0; SplAccount::LEN];
    SplAccount::pack(token_state, &mut token_data)?;

    let mut test = ProgramTest::new("solgov", solgov::ID, processor!(solgov_processor));
    test.add_account(
        governance_mint,
        Account {
            lamports: 1_000_000,
            data: mint_data,
            owner: TOKEN_PROGRAM_ID,
            executable: false,
            rent_epoch: 0,
        },
    );
    test.add_account(
        voter_token_account,
        Account {
            lamports: 1_000_000,
            data: token_data,
            owner: TOKEN_PROGRAM_ID,
            executable: false,
            rent_epoch: 0,
        },
    );

    let mut context = test.start_with_context().await;
    let nonce = 42_u64;
    let creator = context.payer.pubkey();
    let nonce_seed = nonce.to_le_bytes();
    let (dao, _) = solana_sdk::pubkey::Pubkey::find_program_address(
        &[b"dao", creator.as_ref(), &nonce_seed],
        &solgov::ID,
    );
    let (treasury, _) =
        solana_sdk::pubkey::Pubkey::find_program_address(&[b"treasury", dao.as_ref()], &solgov::ID);

    send_instruction(
        &mut context,
        transfer_instruction(&creator, &treasury, 2_000_000_000),
        &[],
    )
    .await?;
    send_instruction(
        &mut context,
        transfer_instruction(&creator, &voter.pubkey(), 1_000_000_000),
        &[],
    )
    .await?;

    send_instruction(
        &mut context,
        Instruction {
            program_id: solgov::ID,
            accounts: solgov::accounts::CreateDao {
                creator,
                governance_mint,
                dao,
                treasury,
                system_program: anchor_lang::system_program::ID,
            }
            .to_account_metas(None),
            data: solgov::instruction::CreateDao {
                nonce,
                name: "Local test DAO".to_owned(),
                quorum: 10,
            }
            .data(),
        },
        &[],
    )
    .await?;

    let proposal_id = 0_u64;
    let proposal_seed = proposal_id.to_le_bytes();
    let (proposal, _) = solana_sdk::pubkey::Pubkey::find_program_address(
        &[b"proposal", dao.as_ref(), &proposal_seed],
        &solgov::ID,
    );
    let recipient = Keypair::new().pubkey();
    send_instruction(
        &mut context,
        Instruction {
            program_id: solgov::ID,
            accounts: solgov::accounts::CreateProposal {
                proposer: creator,
                dao,
                proposal,
                system_program: anchor_lang::system_program::ID,
            }
            .to_account_metas(None),
            data: solgov::instruction::CreateProposal {
                title: "Fund a local test".to_owned(),
                description: "Exercise weighted voting and treasury execution.".to_owned(),
                recipient,
                amount_lamports: 1_000_000,
                voting_period_seconds: 2,
            }
            .data(),
        },
        &[],
    )
    .await?;

    let (vote_record, _) = solana_sdk::pubkey::Pubkey::find_program_address(
        &[b"vote", proposal.as_ref(), voter.pubkey().as_ref()],
        &solgov::ID,
    );
    let cast_vote = Instruction {
        program_id: solgov::ID,
        accounts: solgov::accounts::CastVote {
            voter: voter.pubkey(),
            dao,
            proposal,
            voter_tokens: voter_token_account,
            vote_record,
            system_program: anchor_lang::system_program::ID,
        }
        .to_account_metas(None),
        data: solgov::instruction::CastVote { choice: 0 }.data(),
    };
    send_instruction(&mut context, cast_vote.clone(), &[&voter]).await?;
    assert!(send_instruction(&mut context, cast_vote, &[&voter])
        .await
        .is_err());

    let proposal_account = context
        .banks_client
        .get_account(proposal)
        .await?
        .expect("proposal should have been created");
    let mut proposal_data = proposal_account.data.as_slice();
    let proposal_state = solgov::Proposal::try_deserialize(&mut proposal_data)?;
    assert_eq!(proposal_state.yes_votes, 10);
    assert_eq!(proposal_state.no_votes, 0);
    assert_eq!(proposal_state.quorum, 10);

    let recipient_balance_before = context.banks_client.get_balance(recipient).await?;
    let mut clock = context
        .banks_client
        .get_sysvar::<solana_sdk::clock::Clock>()
        .await?;
    clock.unix_timestamp += 3;
    context.set_sysvar(&clock);
    context
        .warp_to_slot(100)
        .map_err(|error| format!("slot warp failed: {error}"))?;
    let execute = Instruction {
        program_id: solgov::ID,
        accounts: solgov::accounts::ExecuteProposal {
            executor: creator,
            dao,
            proposal,
            treasury,
            recipient,
            system_program: anchor_lang::system_program::ID,
        }
        .to_account_metas(None),
        data: solgov::instruction::ExecuteProposal {}.data(),
    };
    send_instruction(&mut context, execute.clone(), &[]).await?;
    assert_eq!(
        context.banks_client.get_balance(recipient).await? - recipient_balance_before,
        1_000_000
    );
    assert!(send_instruction(&mut context, execute, &[]).await.is_err());

    Ok(())
}
