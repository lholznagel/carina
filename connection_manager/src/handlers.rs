use blockchain_file::peers::{KnownPeers, Peer};
use blockchain_hooks::Hooks;
use blockchain_protocol::enums::status::StatusCodes;
use blockchain_protocol::payload::{NewBlockPayload, PayloadModel, RegisterAckPayload, PossibleBlockPayload, RegisterPayload, PeerRegisteringPayload, ValidateHash};
use blockchain_hooks::EventCodes;
use blockchain_protocol::BlockchainProtocol;

use std::net::{UdpSocket, SocketAddr};
use std::thread;
use std::time::Duration;

pub struct HookHandlers {
    block: u64,
    connected_peers_addr: Vec<String>,
    hash: String,
    validationInProgress: bool
}

impl HookHandlers {
    pub fn new() -> Self {
        Self {
            block: 0,
            connected_peers_addr: Vec::new(),
            hash: String::from(""),
            validationInProgress: false
        }
    }

    fn send_genesis(&self, udp: &UdpSocket) {
        let payload = NewBlockPayload::genesis();

        let message = BlockchainProtocol::new()
            .set_event_code(EventCodes::NewBlock)
            .set_payload(payload)
            .build();

        for peer in self.connected_peers_addr.clone() {
            udp.send_to(
                message.as_slice(),
                peer.parse::<SocketAddr>().unwrap(),
            ).unwrap();
        }
    }
}

impl Hooks for HookHandlers {
    /// # Hole puncher
    ///
    /// - Create a "hole" between to peers
    /// - When a peer registers itself, its IP-Address + Port are saved
    /// - The next peer that registers itself, gets these IP-Address + Port
    /// - The older peer gets the IP-Address + Port of the new peer
    /// - The address of the new peer are saved instead of the old peer
    /// - Both start a ping event to the other peer
    /// - With this technic a connection between two private networks can be accomplished
    ///
    /// In the following graphic, the process is shown
    ///
    /// ```
    ///  1. Register  +--------------+ 2. Register
    ///   +--------->|              |<---------+
    ///   |          | hole puncher |          |
    ///   |    +-----|              |-----+    |
    ///   |    |     +--------------+     |    |
    ///   |    | 3. Send IP+Port of new   |    |
    ///   |    |                          |    |
    ///   |    |                          |    |
    ///   |    |                          |    |
    ///   |    |   4. Send IP+Port of old |    |
    ///   |    v                          v    |
    /// +--------+                      +--------+
    /// |        |--------------------->|        |
    /// | Peer A |      5. Contact      | Peer B |
    /// |        |<---------------------|        |
    /// +--------+                      +--------+
    ///
    /// created with http://asciiflow.com/
    /// ```
    ///
    /// # Example
    ///
    /// - Peer A runs on 192.168.1.5:45678 (on host a)
    /// - Peer B runs on 192.168.1.6:56789 (on host b)
    /// - Peer A registers itself at the hole puncher (some.public.ip.address:45000)
    /// - The hole puncher does not know any peer
    /// - Peer B registers itself at the same hole puncher
    /// - The hole puncher sends the Peer B information to Peer A
    /// - The hole puncher then sends the Peer A information to Peer B
    /// - Peer A and Peer B try to ping each other
    /// - The connection between both networks should be good to go
    ///
    /// Handles a new peer
    fn on_register(&mut self, udp: &UdpSocket, payload_buffer: Vec<u8>, source: String) -> Vec<u8> {
        let register_payload = BlockchainProtocol::<RegisterPayload>::from_vec(payload_buffer);
        let last_peer = KnownPeers::get_latest();
        let mut status = StatusCodes::Ok;

        if last_peer.get_name() == "" {
            status = StatusCodes::NoPeer;
        } else {
            let payload = PeerRegisteringPayload::new().set_addr(source.to_string());
            let message = BlockchainProtocol::new()
                .set_event_code(EventCodes::PeerRegistering)
                .set_payload(payload)
                .build();
            udp.send_to(message.as_slice(), last_peer.get_socket().parse::<SocketAddr>().unwrap()).unwrap();
        }

        KnownPeers::new(Peer::new(register_payload.payload.name(), source.to_string())).save();
        self.connected_peers_addr.push(source.to_string());

        if self.connected_peers_addr.len() >= 3 && self.block == 0 {
            self.send_genesis(&udp);
        }

        let payload = RegisterAckPayload::new().set_addr(String::from(last_peer.get_socket()));
        sending!(format!("ACK_REGISTER | {:?}", payload));
        BlockchainProtocol::new()
            .set_event_code(EventCodes::AckRegister)
            .set_status_code(status)
            .set_payload(payload)
            .build()
    }

    fn on_possible_block(&mut self, udp: &UdpSocket, payload_buffer: Vec<u8>, _: String) -> Vec<u8> {
        let message = BlockchainProtocol::<PossibleBlockPayload>::from_vec(payload_buffer);

        if self.block > message.payload.index {
            self.validationInProgress = false;
        }

        self.block = message.payload.index;
        self.hash = message.payload.hash.clone();

        event!(format!("POSSIBLE_BLOCK | {:?}", message));

        if !self.validationInProgress {
            let mut payload = ValidateHash::new();
            payload.content = message.payload.content;
            payload.index = message.payload.index;
            payload.nonce = message.payload.nonce;
            payload.prev = message.payload.prev;
            payload.timestamp = message.payload.timestamp;

            let message = BlockchainProtocol::new()
                .set_event_code(EventCodes::ValidateHash)
                .set_payload(payload)
                .build();

            for peer in self.connected_peers_addr.clone() {
                self.validationInProgress = true;
                udp.send_to(message.as_slice(), peer.parse::<SocketAddr>().unwrap()).unwrap();
            }
        }

        Vec::new()
    }

    fn on_ping(&self, _: Vec<u8>, _: String) -> Vec<u8> { Vec::new() }
    fn on_pong(&self, _: Vec<u8>, _: String) -> Vec<u8> { Vec::new() }
    fn on_ack_register(&self, _: Vec<u8>, _: String) -> Vec<u8> { Vec::new() }
    fn on_peer_registering(&self, _: Vec<u8>, _: String) -> Vec<u8> { Vec::new() }
    fn on_new_block(&self, _: Vec<u8>, _: String) -> Vec<u8> { Vec::new() }
    fn on_validate_hash(&self, _: Vec<u8>, _: String) -> Vec<u8> { Vec::new() }
    fn on_validated_hash(&self, _: Vec<u8>, _: String) -> Vec<u8> { Vec::new() }
    fn on_found_block(&self, _: Vec<u8>, _: String) -> Vec<u8> { Vec::new() }
}