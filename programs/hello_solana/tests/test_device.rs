use {
    anchor_lang::{
        prelude::Pubkey,
        solana_program::{instruction::Instruction, system_program},
        AccountDeserialize, InstructionData, ToAccountMetas,
    },
    litesvm::LiteSVM,
    solana_keypair::Keypair,
    solana_message::{Message, VersionedMessage},
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
};

#[test]
fn test_device_flow() {
    let program_id = hello_solana::id();
    let payer = Keypair::new();
    let device_id = "sensor-1".to_string();

    let device_pda = Pubkey::find_program_address(
        &[hello_solana::constants::DEVICE_SEED, device_id.as_bytes()],
        &program_id,
    )
    .0;

    let mut svm = LiteSVM::new();
    let bytes = include_bytes!(concat!(
        env!("CARGO_TARGET_TMPDIR"),
        "/../deploy/hello_solana.so"
    ));
    svm.add_program(program_id, bytes).unwrap();
    svm.airdrop(&payer.pubkey(), 1_000_000_000).unwrap();

    // register_device
    let instruction = Instruction::new_with_bytes(
        program_id,
        &hello_solana::instruction::RegisterDevice { device_id: device_id.clone() }.data(),
        hello_solana::accounts::RegisterDevice {
            admin: payer.pubkey(),
            device_authority: payer.pubkey(),
            device: device_pda,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    );

    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[instruction], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&payer]).unwrap();
    let res = svm.send_transaction(tx);
    assert!(res.is_ok(), "register_device failed: {:?}", res);

    let device_account = svm.get_account(&device_pda).unwrap();
    let mut data: &[u8] = &device_account.data;
    let device_state = hello_solana::state::Device::try_deserialize(&mut data).unwrap();
    assert_eq!(device_state.device_id, device_id);
    assert!(device_state.active);
    assert_eq!(device_state.reading_count, 0);

    println!("Device registered: {} (active: {})", device_state.device_id, device_state.active);

    // submit_reading
    let reading_pda = Pubkey::find_program_address(
        &[
            hello_solana::constants::READING_SEED,
            device_pda.as_ref(),
            &0u64.to_le_bytes(),
        ],
        &program_id,
    )
    .0;

    let instruction = Instruction::new_with_bytes(
        program_id,
        &hello_solana::instruction::SubmitReading {
            device_id: device_id.clone(),
            value: 42,
            timestamp: 1_700_000_000,
        }
        .data(),
        hello_solana::accounts::SubmitReading {
            payer: payer.pubkey(),
            device_authority: payer.pubkey(),
            device: device_pda,
            reading: reading_pda,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    );

    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[instruction], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&payer]).unwrap();
    let res = svm.send_transaction(tx);
    assert!(res.is_ok(), "submit_reading failed: {:?}", res);

    let reading_account = svm.get_account(&reading_pda).unwrap();
    let mut data: &[u8] = &reading_account.data;
    let reading_state = hello_solana::state::Reading::try_deserialize(&mut data).unwrap();
    assert_eq!(reading_state.value, 42);
    assert_eq!(reading_state.index, 0);

    let device_account = svm.get_account(&device_pda).unwrap();
    let mut data: &[u8] = &device_account.data;
    let device_state = hello_solana::state::Device::try_deserialize(&mut data).unwrap();
    assert_eq!(device_state.reading_count, 1);
    assert_eq!(device_state.last_value, 42);

    println!("Reading #{} recorded: value={}", reading_state.index, reading_state.value);
    println!("Device now has {} reading(s), last_value={}", device_state.reading_count, device_state.last_value);
}