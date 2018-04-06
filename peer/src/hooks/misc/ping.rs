use carina_hooks::{as_number, ApplicationState, EventCodes};
use carina_protocol::Protocol;
use carina_protocol::payload::EmptyPayload;

use hooks::State;

pub fn ping(state: ApplicationState<State>) {
    info!("Received PING. Answering with PONG.");
    let state_lock = state.state.lock()
        .expect("Locking the mutex should be successful.");
    let answer = Protocol::<EmptyPayload>::new()
        .set_event_code(as_number(EventCodes::Pong))
        .build(&state_lock.nacl);
    state.udp.send_to(&answer, state.source.clone())
        .expect("Sending using UDP should be successful.");
}