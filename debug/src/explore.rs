use blockchain_hooks::{as_number, ApplicationState, as_enum, EventCodes, Hooks, HookRegister};
use blockchain_protocol::Protocol;
use blockchain_protocol::payload::peers::GetPeersAckPayload;
use blockchain_protocol::payload::EmptyPayload;

use clap::ArgMatches;
use futures_cpupool::{CpuFuture, CpuPool};

use std::collections::HashMap;
use std::net::{UdpSocket, SocketAddr};
use std::sync::{Arc, Mutex};
use std::process::exit;
use std::{thread, time};

pub fn execute(hole_puncher: String, args: &ArgMatches) {
    let pool = CpuPool::new_num_cpus();
    let mut threads = Vec::new();

    let wait = args.value_of("WAIT").unwrap().parse::<u64>().expect("Should be able to convert a string to number");

    let state = Arc::new(Mutex::new(ExploreState::new()));

    let request = Protocol::<EmptyPayload>::new()
        .set_event_code(as_number(EventCodes::GetPeers))
        .build();

    let socket = UdpSocket::bind("0.0.0.0:0").expect("Binding an UdpSocket should be successful.");
    socket.send_to(&request, hole_puncher).expect("Sending a request should be successful");

    threads.push(peer(&pool, &state, socket.try_clone().unwrap()));
    thread::sleep(time::Duration::from_secs(wait));
    threads.pop().unwrap().forget();

    let mut success = 0;
    let mut fail = 0;

    let state_lock = state.lock().expect("Locking the mutex should be successful.");
    for (address, value) in &state_lock.peers {
        if state_lock.peers.len() - 1 == value.len() {
            success!("Peer {} knows all peers", address);
            success += 1;
        } else {
            error!("Peer {} does not know all peers", address);
            fail += 1;
        }
    }

    info!("Success: {}, Fail: {}", success, fail);
    exit(0);
}

fn peer(cpu_pool: &CpuPool, state: &Arc<Mutex<ExploreState>>, udp: UdpSocket) -> CpuFuture<bool, ()> {
    let hooks = Hooks::new()
        .set_get_peers_ack(get_peers_ack);
    let mut hook_notification = HookRegister::new(hooks, Arc::clone(&state))
        .get_notification();

    #[allow(unreachable_code)]
    cpu_pool.spawn_fn(move || {
        loop {
            let mut buffer = [0; 65535];

            match udp.recv_from(&mut buffer) {
                Ok((bytes, source)) => {
                    let mut updated_buffer = Vec::new();
                    for i in 0..bytes {
                        updated_buffer.push(buffer[i])
                    }

                    let socket_clone = udp.try_clone().expect("Cloning the socket should be successful.");
                    hook_notification.notify(socket_clone, as_enum(updated_buffer[0]), updated_buffer, source.to_string());
                }
                Err(e) => println!("Error: {:?}", e),
            }
        }

        let res: Result<bool, ()> = Ok(true);
        res
    })
}

pub struct ExploreState {
    peers: HashMap<String, Vec<String>>
}

impl ExploreState {
    /// Creates a new empty instance of ExploreHandler
    pub fn new() -> Self {
        Self {
            peers: HashMap::new()
        }
    }
}

pub fn get_peers_ack(state: ApplicationState<ExploreState>) {
    let message = Protocol::<GetPeersAckPayload>::from_bytes(&state.payload_buffer).expect("Parsing should be successful");
    let mut state_lock = state.state.lock().expect("Locking the mutex should be successful.");

    if !state_lock.peers.contains_key(&state.source) {
        state_lock.peers.insert(state.source, message.payload.peers.clone());

        for address in message.payload.peers {
            let request = Protocol::<EmptyPayload>::new()
                .set_event_code(as_number(EventCodes::GetPeers))
                .build();

            if !address.is_empty() && !state_lock.peers.contains_key(&address) {
                state.udp.send_to(&request, address.parse::<SocketAddr>().unwrap()).expect("Sending a request should be successful");
            }
        }
    }
}