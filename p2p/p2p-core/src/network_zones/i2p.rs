//! I2P Network Zone
//!
//! This module defines the I2P network zone implementation for anonymous networking
//! using the I2P (Invisible Internet Project) network.
//!
//! ### Anonymity
//!
//! This is an anonymous network zone that operates under the following behavior:
//! - The node address is blended into its own address book for privacy
//! - Node ID checking is disabled to preserve anonymity
//! - Used primarily for relaying transactions anonymously
//!
//! ### Addressing
//!
//! The I2P zone uses [`GarlicAddr`] as its address type, representing I2P destinations.
//!

use cuprate_wire::network_address::GarlicAddr;

use crate::{NetZoneAddress, NetworkZone};

impl NetZoneAddress for GarlicAddr {
    type BanID = [u8; 32];

    fn set_port(&mut self, port: u16) {
        self.port = port;
    }

    fn ban_id(&self) -> Self::BanID {
        self.destination()
    }

    fn make_canonical(&mut self) {
        // I2P destinations don't have canonical forms like IP addresses
        // The destination hash is already in canonical form
    }

    fn should_add_to_peer_list(&self) -> bool {
        // For I2P, we generally want to add valid destinations to peer lists
        // Additional validation could be added here if needed
        true
    }
}

/// The I2P network zone.
#[derive(Clone, Copy)]
pub struct I2p;

impl NetworkZone for I2p {
    const NAME: &'static str = "I2p";
    
    /// Don't check node IDs on I2P for anonymity
    const CHECK_NODE_ID: bool = false;
    
    /// Broadcast our own address for better connectivity in the I2P network
    const BROADCAST_OWN_ADDR: bool = true;

    type Addr = GarlicAddr;
}
