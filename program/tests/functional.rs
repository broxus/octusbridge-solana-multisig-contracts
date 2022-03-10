#![cfg(feature = "test-bpf")]

use solana_program_test::{processor, tokio, ProgramTest};

use multisig::Processor;

#[tokio::test]
async fn test() {
    let mut program_test =
        ProgramTest::new("multisig", multisig::id(), processor!(Processor::process));

    // Start Program Test
    let (mut _banks_client, _funder, _recent_blockhash) = program_test.start().await;

    // TODO
}
