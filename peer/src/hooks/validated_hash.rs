use blockchain_hooks::{as_number, ApplicationState, EventCodes};
use blockchain_protocol::BlockchainProtocol;
use blockchain_protocol::payload::{Payload, ValidatedHashPayload, FoundBlockPayload};

use hooks::State;

use std::collections::HashMap;

pub fn on_validated_hash(state: ApplicationState<State>) {
    let message = BlockchainProtocol::<ValidatedHashPayload>::from_bytes(&state.payload_buffer)
        .expect("Parsing the protocol should be successful.");
    let mut state_lock = state.state.lock()
        .expect("Locking the mutex should be successful.");

    state_lock.hashes.push(message.payload.hash);

    if state_lock.hashes.len() == state_lock.peers.len() {
        let mut hashes = HashMap::new();

        for hash in state_lock.hashes.clone() {
            let updated_value = match hashes.get(&hash) {
                Some(current_val)   => current_val + 1,
                None                => 1
            };

            hashes.insert(hash, updated_value);
        }

        let mut result: (String, u64) = (String::from(""), 0);
        for (key, value) in hashes {
            if result.1 == 0 || value > result.1 {
                result.0 = key;
                result.1 = value;
            }
        }

        state_lock.hashes = Vec::new();
        state_lock.current_block.hash = result.0;

        let mut payload = FoundBlockPayload::new();
        payload.content = state_lock.current_block.content.clone();
        payload.index = state_lock.current_block.index;
        payload.nonce = state_lock.current_block.nonce;
        payload.prev = state_lock.current_block.prev.clone();
        payload.timestamp = state_lock.current_block.timestamp;
        payload.hash = state_lock.current_block.hash.clone();

        let message = BlockchainProtocol::new()
            .set_event_code(as_number(EventCodes::FoundBlock))
            .set_payload(payload)
            .build();

        for (peer, _) in state_lock.peers.clone() {
            state.udp.send_to(message.as_slice(), peer).unwrap();
        }
    }
}