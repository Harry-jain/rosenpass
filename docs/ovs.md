# Open vSwitch (OVS) Support in Rosenpass

Rosenpass supports managing WireGuard interfaces by directly attaching them to an Open vSwitch (OVS) bridge. This feature simplifies network integration by letting Rosenpass handle creating the WireGuard interface and adding it as a port to an existing (or dynamically created) OVS bridge.

## Configuration

To use the OVS network backend, add the `[network]` section to your `config.toml`:

```toml
public_key = "rp-public-key"
secret_key = "rp-secret-key"
listen = ["0.0.0.0:9999"]
verbosity = "Verbose"

[network]
backend = "ovs"
bridge = "br0"
interface = "wg0"

[[peers]]
public_key = "peer-public-key"
endpoint = "127.0.0.1:10000"
key_out = "key-out.txt"
```

In this example, Rosenpass will:
1. Check if `ovs-vsctl` is installed.
2. Create `br0` if it doesn't exist.
3. Bring up a WireGuard interface named `wg0`.
4. Attach `wg0` to `br0`.

Upon stopping (e.g. receiving `SIGTERM` or `SIGINT`), Rosenpass cleanly detaches `wg0` from `br0` and removes the `wg0` interface.

## Demo / Testing

You can use the included `demo-ovs.sh` script to verify behavior. The demo will:
1. Create a demo configuration with the `[network]` block.
2. Generate dummy keys.
3. Run `rosenpass exchange-config` in the background.
4. Automatically initialize the OVS attach.
5. Terminate the process to show clean teardown of the network interfaces.
