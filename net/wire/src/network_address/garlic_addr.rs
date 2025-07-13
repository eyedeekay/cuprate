//! Garlic address
//!
//! This module define I2P Destination addresses
//!

use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

use cuprate_epee_encoding::{error::*, read_epee_value, write_field, EpeeObject, EpeeObjectBuilder};

/// An I2P garlic address.
///
/// This represents an I2P destination address used for anonymous networking.
/// I2P destinations are represented as Base64-encoded strings that contain
/// the public key and certificate information.
#[derive(Clone, Debug, Copy, PartialEq, Eq, Hash)]
pub struct GarlicAddr {
    /// The I2P destination encoded as Base64, truncated to 32 bytes for storage efficiency
    /// The full destination would be much larger (~387+ bytes), but we store a hash/truncated version
    destination: [u8; 32],
    /// Virtual port of the service
    pub port: u16,
}

impl GarlicAddr {
    /// Creates a new `GarlicAddr` from a destination hash and port.
    pub const fn new(destination: [u8; 32], port: u16) -> Self {
        Self { destination, port }
    }

    /// Returns the destination hash.
    pub const fn destination(&self) -> [u8; 32] {
        self.destination
    }

    /// Returns the port.
    pub const fn port(&self) -> u16 {
        self.port
    }
}

impl Display for GarlicAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // Display as a truncated Base64-like representation with port
        write!(
            f,
            "{}...{}:{}",
            base64::encode(&self.destination[..8]),
            base64::encode(&self.destination[24..]),
            self.port
        )
    }
}

impl FromStr for GarlicAddr {
    type Err = std::io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Parse format: "base64destination:port" or "truncated_display_format"
        let parts: Vec<&str> = s.rsplitn(2, ':').collect();
        if parts.len() != 2 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid garlic address format",
            ));
        }

        let port = parts[0].parse::<u16>().map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid port")
        })?;

        let dest_str = parts[1];
        
        // For now, create a simple hash of the destination string
        // In a real implementation, you'd decode the Base64 I2P destination
        let mut destination = [0u8; 32];
        let hash = std::collections::hash_map::DefaultHasher::new();
        use std::hash::{Hash, Hasher};
        dest_str.hash(&mut hash.clone());
        let hash_val = hash.finish();
        destination[..8].copy_from_slice(&hash_val.to_le_bytes());
        
        Ok(Self::new(destination, port))
    }
}

impl EpeeObjectBuilder<GarlicAddr> for () {
    fn add_field<T: EpeeObject>(&mut self, name: &str, t: T) -> Result<()> {
        Err(EpeeError::Format("Garlic address builder not implemented"))
    }

    fn finish(self) -> Result<GarlicAddr> {
        Err(EpeeError::Format("Garlic address builder not implemented"))
    }
}

impl EpeeObject for GarlicAddr {
    type Builder = ();

    fn number_of_fields(&self) -> u64 {
        2
    }

    fn write_fields<B: cuprate_epee_encoding::bytes::BufMut>(
        self,
        w: &mut B,
    ) -> cuprate_epee_encoding::Result<()> {
        // Write destination as bytes
        write_field(&self.destination.to_vec(), "destination", w)?;
        // Write port
        write_field(&self.port, "port", w)?;
        Ok(())
    }
}