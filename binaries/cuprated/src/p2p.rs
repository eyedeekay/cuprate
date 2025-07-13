//! P2P
//!
//! Will handle initiating the P2P and contains a protocol request handler.

use std::convert::From;

use futures::{FutureExt, TryFutureExt};
use tokio::sync::oneshot::{self, Sender};
use tower::{Service, ServiceExt};

use cuprate_blockchain::service::{BlockchainReadHandle, BlockchainWriteHandle};
use cuprate_consensus::BlockchainContextService;
use cuprate_p2p::{config::TransportConfig, NetworkInterface, P2PConfig};
use cuprate_p2p_core::{client::InternalPeerID, transports::Tcp, ClearNet, I2p, NetworkZone, Transport};
use cuprate_p2p_transport::I2pTransport;
use cuprate_txpool::service::{TxpoolReadHandle, TxpoolWriteHandle};
use cuprate_types::blockchain::BlockchainWriteRequest;

use crate::{
    blockchain,
    config::Config,
    constants::PANIC_CRITICAL_SERVICE_ERROR,
    txpool::{self, IncomingTxHandler},
};

mod core_sync_service;
mod network_address;
pub mod request_handler;

pub use network_address::CrossNetworkInternalPeerId;

/// This struct collect all supported and optional network zone interfaces.
pub struct NetworkInterfaces {
    /// Mandatory clearnet network interface
    pub clearnet_network_interface: NetworkInterface<ClearNet>,
    /// Optional I2P network interface
    pub i2p_network_interface: Option<NetworkInterface<I2p>>,
    // ...one can dream for more!
}

impl NetworkInterfaces {
    pub const fn new(
        clearnet_network_interface: NetworkInterface<ClearNet>,
        i2p_network_interface: Option<NetworkInterface<I2p>>,
    ) -> Self {
        Self {
            clearnet_network_interface,
            i2p_network_interface,
        }
    }
}

/// Initialize all P2P network zones. Returning a [`NetworkInterfaces`] collection and
/// a [`Vec<Sender<IncomingTxHandler>>`] for propagating the tx handler.
pub async fn initialize_zones_p2p(
    config: &Config,
    context_svc: BlockchainContextService,
    mut blockchain_write_handle: BlockchainWriteHandle,
    mut blockchain_read_handle: BlockchainReadHandle,
    txpool_write_handle: TxpoolWriteHandle,
    txpool_read_handle: TxpoolReadHandle,
) -> (NetworkInterfaces, Vec<Sender<IncomingTxHandler>>) {
    // Start TCP clearnet P2P.
    let (clearnet, incoming_tx_handler_tx) = start_zone_p2p::<ClearNet, Tcp>(
        blockchain_read_handle.clone(),
        context_svc.clone(),
        txpool_read_handle.clone(),
        config.clearnet_p2p_config(),
        (&config.p2p.clear_net).into(),
    )
    .await
    .unwrap();

    // Start I2P P2P if enabled
    let mut tx_handler_subscribers = vec![incoming_tx_handler_tx];
    let i2p_interface = if config.p2p.i2p.enable {
        info!("Starting I2P P2P network zone");
        
        match start_zone_p2p::<I2p, I2pTransport>(
            blockchain_read_handle.clone(),
            context_svc.clone(),
            txpool_read_handle.clone(),
            config.i2p_p2p_config(),
            (&config.p2p.i2p).into(),
        )
        .await
        {
            Ok((i2p, i2p_tx_handler)) => {
                info!("Successfully started I2P network zone");
                tx_handler_subscribers.push(i2p_tx_handler);
                Some(i2p)
            }
            Err(e) => {
                warn!("Failed to start I2P network zone: {}", e);
                info!("Continuing without I2P support");
                None
            }
        }
    } else {
        debug!("I2P network zone disabled in configuration");
        None
    };

    // Create network interface collection
    let network_interfaces = NetworkInterfaces::new(clearnet, i2p_interface);
    (network_interfaces, tx_handler_subscribers)
}

/// Starts the P2P network zone, returning a [`NetworkInterface`] to interact with it.
///
/// A [`oneshot::Sender`] is also returned to provide the [`IncomingTxHandler`], until this is provided network
/// handshakes can not be completed.
pub async fn start_zone_p2p<N, T>(
    blockchain_read_handle: BlockchainReadHandle,
    blockchain_context_service: BlockchainContextService,
    txpool_read_handle: TxpoolReadHandle,
    config: P2PConfig<N>,
    transport_config: TransportConfig<N, T>,
) -> Result<(NetworkInterface<N>, Sender<IncomingTxHandler>), tower::BoxError>
where
    N: NetworkZone,
    T: Transport<N>,
    N::Addr: borsh::BorshDeserialize + borsh::BorshSerialize,
    CrossNetworkInternalPeerId: From<InternalPeerID<<N as NetworkZone>::Addr>>,
{
    let (incoming_tx_handler_tx, incoming_tx_handler_rx) = oneshot::channel();

    let request_handler_maker = request_handler::P2pProtocolRequestHandlerMaker {
        blockchain_read_handle,
        blockchain_context_service: blockchain_context_service.clone(),
        txpool_read_handle,
        incoming_tx_handler: None,
        incoming_tx_handler_fut: incoming_tx_handler_rx.shared(),
    };

    Ok((
        cuprate_p2p::initialize_network::<N, T, _, _>(
            request_handler_maker.map_response(|s| s.map_err(Into::into)),
            core_sync_service::CoreSyncService(blockchain_context_service),
            config,
            transport_config,
        )
        .await?,
        incoming_tx_handler_tx,
    ))
}
