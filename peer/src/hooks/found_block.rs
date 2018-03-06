use blockchain_hooks::ApplicationState;
use blockchain_protocol::BlockchainProtocol;
use blockchain_protocol::payload::FoundBlockPayload;

use hooks::State;

use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub fn on_found_block(state: ApplicationState<State>) {
    let message = BlockchainProtocol::<FoundBlockPayload>::from_bytes(&state.payload_buffer)
        .expect("Parsing the protocol should be successful.");
    {
        let state_lock = state.state.lock()
            .expect("Locking the mutex should be successful.");

        if !Path::new(&state_lock.storage).exists() {
            fs::create_dir(&state_lock.storage).unwrap();
        }
    }

    save_file(message.payload, state);
}

fn save_file(block: FoundBlockPayload, state: ApplicationState<State>) {
    let state_lock = state.state.lock()
        .expect("Locking the mutex should be successful.");

    let mut filename = String::from("");

    for i in 0..16 {
        filename = filename + &block.hash.chars().nth(48 + i).unwrap().to_string();
    }

    if !Path::new(&filename).exists() {
        info!("Saving new block to disk.");
        let mut file = File::create(format!("{}/{}", state_lock.storage, filename))
            .expect("Could not create block file.");
        let mut file_last = File::create(format!("{}/last", state_lock.storage))
            .expect("Could not create block file.");

        let content = String::from(
            format!(
                "{}\n{}\n{}\n{}\n{}\n{}", 
                block.index,
                block.content, 
                block.timestamp,
                block.nonce,
                block.prev,
                block.hash
            ));

        file.write_all(content.clone().as_bytes())
            .expect("Error writing block information into file.");
        file_last.write_all(content.clone().as_bytes())
            .expect("Error writing block information into file.");
    }
}