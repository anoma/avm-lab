//! `avm-node` — distributed AVM runtime binary.
//!
//! Launches an AVM node that connects to peers over TCP and can run the
//! distributed ping-pong demo to exercise cross-node object calls.
//!
//! # Usage
//!
//! Start the beta node first:
//! ```text
//! avm-node --name beta --port 9002 --peer alpha:9001 --demo
//! ```
//! Then start alpha:
//! ```text
//! avm-node --name alpha --port 9001 --peer beta:9002 --demo
//! ```

use avm_core::itree::ret;
use avm_core::types::{ObjectId, Val};
use clap::Parser;
use tracing::{info, warn};

mod directory;
mod node;
mod protocol;
mod remote_call;
mod sse;
mod transport;

use node::{Node, NodeConfig};

/// Top-level error type for the node binary.
#[derive(Debug, thiserror::Error)]
pub enum NodeError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("serialization error: {0}")]
    Serialize(serde_json::Error),

    #[error("deserialization error: {0}")]
    Deserialize(serde_json::Error),

    #[error("frame too large: {0} bytes")]
    FrameTooLarge(usize),

    #[error("interpret error: {0}")]
    Interpret(String),
}

/// Command-line arguments for `avm-node`.
#[derive(Parser, Debug)]
#[command(name = "avm-node", about = "Distributed AVM runtime node")]
struct Args {
    /// This node's name (used as its `MachineId`).
    #[arg(long, default_value = "node")]
    name: String,

    /// TCP port to listen on.
    #[arg(long, default_value_t = 9001)]
    port: u16,

    /// Peer addresses to connect to, e.g. `--peer alpha:9001`.
    /// May be repeated for multiple peers.
    #[arg(long = "peer")]
    peers: Vec<String>,

    /// Run the distributed ping-pong demo.
    #[arg(long)]
    demo: bool,

    /// Number of ping-pong rounds for the demo (only used by alpha).
    #[arg(long, default_value_t = 3)]
    rounds: u64,

    /// TCP port for the SSE HTTP server (`GET /events`, `GET /health`).
    /// Defaults to 8080. Set to 0 to disable the SSE server.
    #[arg(long, default_value_t = 8080)]
    sse_port: u16,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let args = Args::parse();

    // Assign a unique node prefix from the port number (lower 16 bits).
    let node_prefix = args.port;

    let sse_port = if args.sse_port == 0 {
        None
    } else {
        Some(args.sse_port)
    };

    let config = NodeConfig {
        name: args.name.clone(),
        port: args.port,
        peers: args.peers.clone(),
        node_prefix,
        sse_port,
    };

    if args.demo {
        run_demo(args, config).await?;
    } else {
        // Plain node: just listen and serve remote calls indefinitely.
        let registry = avm_core::vm::BehaviorRegistry::new();
        let node = Node::new(config, registry);
        let _running = node.start().await?;
        info!(name = %args.name, "node running, press Ctrl-C to stop");
        tokio::signal::ctrl_c().await?;
    }

    Ok(())
}

/// Run the distributed ping-pong demo.
///
/// - **beta**: creates the pong object locally, broadcasts it, then serves.
/// - **alpha**: creates the ping object locally, waits for pong's location,
///   then runs the orchestrator that kicks off the exchange.
async fn run_demo(args: Args, config: NodeConfig) -> Result<(), NodeError> {
    use avm_examples::ping_pong::ping_pong_registry;

    let registry = ping_pong_registry();
    let node = Node::new(config, registry);
    let running = node.start().await?;

    // Give the listener a moment to bind before peers try to connect.
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    match args.name.as_str() {
        "beta" => {
            info!("beta: creating pong object");

            // ObjectId for pong — chosen to not collide with alpha's IDs.
            // Alpha uses prefix 9001 << 48, beta uses prefix 9002 << 48.
            let pong_id = ObjectId(u64::from(args.port) << 48);

            running.register_object(pong_id, "pong");
            info!(%pong_id, "beta: registered pong locally");

            // Wait until alpha is connected before announcing.
            if !args.peers.is_empty() {
                let peer_names: Vec<&str> = args
                    .peers
                    .iter()
                    .map(|p| p.split(':').next().unwrap_or(p.as_str()))
                    .collect();
                info!("beta: waiting for alpha to connect...");
                running.wait_for_peers(&peer_names).await;
                info!("beta: alpha connected");
            }

            running.announce_object(pong_id);
            info!(%pong_id, "beta: announced pong to peers");

            // Serve indefinitely.
            info!("beta: serving pong, waiting for calls...");
            tokio::signal::ctrl_c().await?;
            info!("beta: shutting down");
        }

        "alpha" => {
            // IDs are keyed by the node's port number in the high bits.
            let local_ping_id = ObjectId(u64::from(args.port) << 48);
            let remote_pong_id = ObjectId(u64::from(9002u16) << 48);

            running.register_object(local_ping_id, "ping");
            info!(%local_ping_id, "alpha: registered ping locally");

            // Wait for beta's peer connection and pong directory entry.
            if !args.peers.is_empty() {
                let peer_names: Vec<&str> = args
                    .peers
                    .iter()
                    .map(|p| p.split(':').next().unwrap_or(p.as_str()))
                    .collect();
                info!("alpha: waiting for beta to connect...");
                running.wait_for_peers(&peer_names).await;
                info!("alpha: beta connected");
            }

            info!("alpha: waiting for pong object to appear in directory...");
            running.wait_for_object(remote_pong_id).await;
            info!(%remote_pong_id, "alpha: pong found on beta");

            // Build the initial call: send to ping, which will call pong remotely.
            let max_count = args.rounds;

            // Pass a factory closure — the `ITree` is built on the interpreter thread,
            // so neither it nor its `!Send` continuations cross a thread boundary.
            info!("alpha: starting ping-pong exchange ({max_count} rounds)");
            let result = running
                .run_program(move || {
                    let initial_msg = Val::list(vec![
                        Val::str("Ping"),
                        Val::Nat(0),
                        Val::Nat(max_count),
                        Val::ObjectRef(remote_pong_id),
                    ]);
                    avm_core::avm_do! {
                        let response <- avm_core::itree::trigger(
                            avm_core::instruction::call(local_ping_id, initial_msg)
                        );
                        ret(response)
                    }
                })
                .await?;

            info!("alpha: exchange complete!");
            info!("alpha: result = {}", result.value);
            info!("alpha: trace has {} events", result.trace.len());
            for entry in &result.trace {
                info!("  {entry:?}");
            }
        }

        other => {
            warn!(name = %other, "demo mode only supports 'alpha' and 'beta' node names");
        }
    }

    Ok(())
}
