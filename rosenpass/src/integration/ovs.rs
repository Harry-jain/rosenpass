//! Open vSwitch (OVS) backend integration
//!
//! Provides functions to create a WireGuard interface,
//! attach it to an OVS bridge, and tear it down cleanly.

use std::process::Command;
use anyhow::{anyhow, Context, Result};
use log::{info, warn};

/// Check if Open vSwitch is installed by running `ovs-vsctl --version`
pub fn ovs_installed() -> bool {
    Command::new("ovs-vsctl")
        .arg("--version")
        .output()
        .is_ok()
}

/// Creates a WireGuard interface by invoking `ip link add`
pub fn create_wg_interface(iface: &str) -> Result<()> {
    info!("Creating WireGuard interface: {}", iface);
    let status = Command::new("ip")
        .args(["link", "add", iface, "type", "wireguard"])
        .status()
        .context("Failed to execute `ip link add`")?;

    if !status.success() {
        return Err(anyhow!("Failed to create wireguard interface {}. Is `ip` installed and are you running as root?", iface));
    }
    
    // Bring interface up
    let status_up = Command::new("ip")
        .args(["link", "set", "up", "dev", iface])
        .status()
        .context("Failed to execute `ip link set up`")?;
        
    if !status_up.success() {
        warn!("Failed to bring interface {} up, but returning success for creation.", iface);
    }
    
    Ok(())
}

/// Deletes a WireGuard interface
pub fn delete_wg_interface(iface: &str) -> Result<()> {
    info!("Deleting WireGuard interface: {}", iface);
    let status = Command::new("ip")
        .args(["link", "del", iface])
        .status()
        .context("Failed to execute `ip link del`")?;

    if !status.success() {
        warn!("Failed to delete wireguard interface {}. It may have already been deleted.", iface);
    }
    Ok(())
}

/// Ensures the given OVS bridge exists; creates it otherwise (optional enhancement)
pub fn create_ovs_bridge(bridge: &str) -> Result<()> {
    if !ovs_installed() {
        return Err(anyhow!("Open vSwitch (ovs-vsctl) is not installed"));
    }

    info!("Ensuring OVS bridge exists: {}", bridge);
    let status = Command::new("ovs-vsctl")
        .args(["--may-exist", "add-br", bridge])
        .status()
        .context("Failed to execute `ovs-vsctl add-br`")?;

    if !status.success() {
        return Err(anyhow!("Failed to create or verify OVS bridge {}", bridge));
    }
    Ok(())
}

/// Attaches a given interface to the OVS bridge
pub fn add_interface_to_bridge(bridge: &str, iface: &str) -> Result<()> {
    if !ovs_installed() {
        return Err(anyhow!("Open vSwitch (ovs-vsctl) is not installed"));
    }

    info!("Attaching {} -> {}", iface, bridge);
    let status = Command::new("ovs-vsctl")
        .args(["--may-exist", "add-port", bridge, iface])
        .status()
        .context("Failed to execute `ovs-vsctl add-port`")?;

    if !status.success() {
        return Err(anyhow!("Failed to add interface {} to OVS bridge {}", iface, bridge));
    }
    Ok(())
}

/// Detaches a given interface from the OVS bridge
pub fn remove_interface_from_bridge(bridge: &str, iface: &str) -> Result<()> {
    if !ovs_installed() {
        return Err(anyhow!("Open vSwitch (ovs-vsctl) is not installed"));
    }

    info!("Detaching {} <- {}", iface, bridge);
    let status = Command::new("ovs-vsctl")
        .args(["--if-exists", "del-port", bridge, iface])
        .status()
        .context("Failed to execute `ovs-vsctl del-port`")?;

    if !status.success() {
        warn!("Failed to remove interface {} from OVS bridge {}. It may have already been removed.", iface, bridge);
    }
    Ok(())
}

/// OVS integrated setup: creates `wg` interface, creates bridge if missing, attaches.
pub fn setup_ovs_backend(bridge: &str, iface: &str) -> Result<()> {
    if !ovs_installed() {
        return Err(anyhow!("Open vSwitch is not installed. Cannot use `ovs` network backend."));
    }
    // Auto-create bridge if missing (adds competitive edge)
    create_ovs_bridge(bridge)?;
    // Create WireGuard interface
    create_wg_interface(iface)?;
    // Attach to bridge
    add_interface_to_bridge(bridge, iface)?;
    Ok(())
}

/// OVS integrated teardown: detaches bridge and deletes `wg` interface.
pub fn teardown_ovs_backend(bridge: &str, iface: &str) -> Result<()> {
    if !ovs_installed() {
        return Ok(()); // Nothing to do
    }
    // Ignoring errors in detach since it's teardown
    let _ = remove_interface_from_bridge(bridge, iface);
    // Delete WireGuard interface
    let _ = delete_wg_interface(iface);
    Ok(())
}
