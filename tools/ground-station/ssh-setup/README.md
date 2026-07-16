# SSH Setup Utility

A lightweight utility for automating SSH key distribution and server configuration across a local network. This tool consists of a master server that distributes SSH configuration to slave nodes, enabling secure one-way SSH access from the master to all configured slaves.

## What It Does

This utility automates the process of:
1. **SSH Key Distribution**: Automatically distributes the master's SSH public key to slave nodes
2. **SSH Server Setup**: Ensures SSH servers are installed and running on slave nodes
3. **Cross-Platform Support**: Works across multiple Linux distributions (Debian, Ubuntu, Fedora, CentOS, Arch, etc.)
4. **Network Discovery**: Automatically detects network interfaces and IP addresses
5. **Zero-Configuration**: Minimal setup required - just run the server and execute one command on each slave

## Quick Start

### 1. Start the Master Server

On your master/control machine:

```bash
cd utils/ssh-setup
python3 server.py
```

The server will:
- Auto-detect your primary network IP address
- Use existing SSH keys or generate new ones if needed
- Start an HTTP server on port 8000
- Display the curl command to run on slave nodes

### 2. Configure Slave Nodes

On each slave node you want to configure, run:

```bash
curl -sSL http://MASTER_IP:8000/install | python3
```

Replace `MASTER_IP` with the actual IP address shown by the server.

## How It Works

### Master Server (`server.py`)

1. **Network Detection**: Automatically finds the primary network interface and IP address
2. **SSH Key Management**: Uses existing SSH keys from `~/.ssh/id_rsa.pub` or generates new ones
3. **Template Processing**: Serves a customized `bootstrap.py` script with:
   - Master's IP address
   - Master's SSH public key
   - Slave's IP address (detected from HTTP request)

### Bootstrap Script (`bootstrap.py`)

1. **OS Detection**: Identifies the Linux distribution and package manager
2. **SSH Server Installation**: Installs OpenSSH server if not present
3. **Service Configuration**: Ensures SSH service is enabled and running
4. **Key Distribution**: Adds master's public key to user's `~/.ssh/authorized_keys`
5. **mDNS Support**: Checks for Avahi service to provide hostname-based access

## Supported Distributions

The utility supports multiple Linux distributions through package manager detection:

- **Debian-based**: Ubuntu, Debian, Pop!_OS, Linux Mint
- **Red Hat-based**: Fedora, CentOS, RHEL, Rocky Linux
- **Arch-based**: Arch Linux, Manjaro

### Prerequisites

- All nodes must be on the same local network

**Master Server Requirements:**
- Python 3.6+
- `ssh-keygen` command available
- Network access to slave nodes

**Slave Node Requirements:**
- Python 3.6+
- `sudo` privileges (for package installation and service management)
- `systemctl` command available
- Network connectivity to master server

### Customization

**Adding New Distributions:**
Edit the `DISTRO_CONFIG` dictionary in `bootstrap.py`:

```python
DISTRO_CONFIG = {
    "your_distro": {
        "pkg_manager": "your_pkg_manager",
        "update_cmd": ["update"],
        "install_cmd": ["install", "-y"],
        "ssh_pkg": "openssh-server",
        "ssh_svc": "ssh",
    }
}
```

**Changing Server Port:**
Modify the `SERVER_PORT` variable in `server.py`.

**Custom SSH Configuration:**
Edit the `setup_user_ssh()` function in `bootstrap.py` to add additional SSH options.

## Example Usage

```bash
# On master (192.168.1.100)
$ cd utils/ssh-setup
$ python3 server.py
--- Bootstrap Server Running ---
Auto-detected Master IP: 192.168.1.100
Server listening on http://0.0.0.0:8000

To install on a slave node, run:
curl -sSL http://192.168.1.100:8000/install | python3

# On slave (192.168.1.101)
$ curl -sSL http://192.168.1.100:8000/install | python3
[bootstrap] Performing pre-flight checks...
[bootstrap] Detected distribution: ubuntu
[bootstrap] Setting up user-specific SSH configuration for user: ubuntu
[bootstrap] SSH server (sshd) is already installed.
[bootstrap] Bootstrap complete! Slave node is ready.
[bootstrap] Master can connect to this slave using: ssh ubuntu@192.168.1.101

# Test connection from master
$ ssh ubuntu@192.168.1.101
```
