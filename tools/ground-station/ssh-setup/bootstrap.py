#!/usr/bin/env python3

import os
import sys
import subprocess
import shutil
import tempfile

# This script bootstraps a slave node for one-way SSH access from a master server.
# It must be run as the user you wish to configure (e.g., `pi`, `ubuntu`).
# It detects the OS, installs the SSH server if missing, configures SSH access
# for the user, and ensures the system-wide SSH daemon is running correctly.

# --- TEMPLATE VARIABLES ---
# These would be replaced by your templating engine before the script is run.
MASTER_IP = "{{MASTER_IP}}"          # e.g. "192.168.88.10"
MASTER_SSH_PUBKEY = "{{MASTER_SSH_PUBKEY}}"  # e.g. "ssh-ed25519 AAAA... comment"
SLAVE_IP = "{{SLAVE_IP}}"            # e.g. "192.168.88.11"


# --- DISTRIBUTION CONFIGURATION (EXTENSIBLE) ---
# To add support for a new distribution, add its ID (from /etc/os-release)
# to the appropriate family or create a new family archetype.

# 1. Define the archetypes for distro families.
DEBIAN_LIKE = {
    "pkg_manager": "apt-get",
    "update_cmd": ["update"],
    "install_cmd": ["install", "-y"],
    "ssh_pkg": "openssh-server",
    "ssh_svc": "ssh",
}
FEDORA_LIKE = {
    "pkg_manager": "dnf",
    "update_cmd": [], # dnf doesn't typically require a separate update command
    "install_cmd": ["install", "-y"],
    "ssh_pkg": "openssh-server",
    "ssh_svc": "sshd",
}
ARCH_LIKE = {
    "pkg_manager": "pacman",
    "update_cmd": ["-Sy"], # Sync and update repositories
    "install_cmd": ["-S", "--noconfirm"],
    "ssh_pkg": "openssh",
    "ssh_svc": "sshd",
}

# 2. Map OS IDs to their family archetypes.
DISTRO_CONFIG = {
    "debian":   DEBIAN_LIKE,
    "ubuntu":   DEBIAN_LIKE,
    "pop":      DEBIAN_LIKE, # Pop!_OS
    "linuxmint":DEBIAN_LIKE, # Linux Mint
    "fedora":   FEDORA_LIKE,
    "centos":   FEDORA_LIKE,
    "rhel":     FEDORA_LIKE, # Red Hat Enterprise Linux
    "rocky":    FEDORA_LIKE, # Rocky Linux
    "arch":     ARCH_LIKE,
    "manjaro":  ARCH_LIKE,
}

# --- UTILS ---

def log(message):
    """Prints a standard log message."""
    print(f"[bootstrap] {message}")

def err(message):
    """Prints a colorized error message to stderr."""
    print(f"\033[0;31m[bootstrap][ERROR] {message}\033[0m", file=sys.stderr)

def require_filled(name, value):
    """Checks if a template variable has been filled."""
    if not value or "{{" in value or "}}" in value:
        err(f"Template variable {name} is not set.")
        sys.exit(1)

def require_cmd(command):
    """Checks if a command exists in the system's PATH."""
    if not shutil.which(command):
        err(f"Required command '{command}' not found in PATH.")
        sys.exit(1)

def run_command(command_list, use_sudo=False, check=True):
    """Runs a command, optionally with sudo, and handles errors."""
    if use_sudo:
        require_cmd("sudo")
        command_list.insert(0, "sudo")

    log(f"Running command: {' '.join(command_list)}")
    try:
        result = subprocess.run(
            command_list, check=check, capture_output=True, text=True
        )
        return result
    except subprocess.CalledProcessError as e:
        err(f"Command failed with exit code {e.returncode}: {' '.join(command_list)}")
        err(f"STDOUT: {e.stdout.strip()}")
        err(f"STDERR: {e.stderr.strip()}")
        sys.exit(1)
    except FileNotFoundError:
        err(f"Command not found: {command_list[0]}")
        sys.exit(1)

def get_distro_info():
    """
    Detects the Linux distribution by reading /etc/os-release and returns
    the corresponding configuration from DISTRO_CONFIG.
    """
    if not os.path.exists("/etc/os-release"):
        err("Cannot determine OS: /etc/os-release not found. This script requires a standard Linux system.")
        return None

    with open("/etc/os-release") as f:
        lines = f.readlines()

    os_info = {}
    for line in lines:
        if "=" in line:
            key, value = line.strip().split("=", 1)
            os_info[key] = value.strip('"')

    distro_id = os_info.get("ID")
    id_like = os_info.get("ID_LIKE", "").split()

    # Check primary ID first, then fall back to ID_LIKE for derivatives.
    if distro_id in DISTRO_CONFIG:
        log(f"Detected distribution: {distro_id}")
        return DISTRO_CONFIG[distro_id]

    for id_val in id_like:
        if id_val in DISTRO_CONFIG:
            log(f"Detected compatible distribution: {id_val} (via ID_LIKE)")
            return DISTRO_CONFIG[id_val]

    err(f"Unsupported Linux distribution: '{distro_id}'. Supported: {list(DISTRO_CONFIG.keys())}")
    return None


# --- SCRIPT LOGIC ---

def setup_user_ssh():
    """Configures the user's ~/.ssh/authorized_keys file."""
    log(f"Setting up user-specific SSH configuration for user: {os.getlogin()}")

    home_dir = os.path.expanduser("~")
    ssh_dir = os.path.join(home_dir, ".ssh")
    auth_keys_path = os.path.join(ssh_dir, "authorized_keys")

    os.makedirs(ssh_dir, mode=0o700, exist_ok=True)

    if not os.path.exists(auth_keys_path):
        open(auth_keys_path, 'a').close()
    os.chmod(auth_keys_path, 0o600)

    try:
        key_material = " ".join(MASTER_SSH_PUBKEY.split()[:2])
    except IndexError:
        err("MASTER_SSH_PUBKEY appears to be malformed.")
        sys.exit(1)

    key_line = MASTER_SSH_PUBKEY # No 'from=' IP restriction

    lines = []
    key_found = False
    with open(auth_keys_path, "r") as f:
        for line in f:
            line = line.strip()
            if not line: continue
            if key_material in line:
                lines.append(key_line)
                key_found = True
            else:
                lines.append(line)

    if not key_found:
        lines.append(key_line)
        log("Adding master's key to user's authorized_keys.")
    else:
        log("Master's key material found. Ensured line is up-to-date.")

    with tempfile.NamedTemporaryFile("w", dir=ssh_dir, delete=False) as tmp:
        tmp.write("\n".join(lines) + "\n")
        temp_path = tmp.name

    os.replace(temp_path, auth_keys_path)
    os.chmod(auth_keys_path, 0o600)
    log("User-specific SSH setup complete.")

def setup_system_ssh(distro_info):
    """
    Ensures the SSH server package is installed, configured, and running.
    """
    log("Checking system SSH service status.")
    ssh_service = distro_info['ssh_svc']

    # 1. Install SSH server if it's not present
    if not shutil.which("sshd"):
        log("SSH server (sshd) not found. Attempting installation...")
        pkg_manager = distro_info['pkg_manager']

        if distro_info['update_cmd']:
            run_command([pkg_manager] + distro_info['update_cmd'], use_sudo=True)

        install_command = [pkg_manager] + distro_info['install_cmd'] + [distro_info['ssh_pkg']]
        run_command(install_command, use_sudo=True)

        require_cmd("sshd") # Verify installation was successful
        log(f"Successfully installed {distro_info['ssh_pkg']}.")
    else:
        log("SSH server (sshd) is already installed.")

    # 2. Configure and enable the service
    host_keys_exist = any(f.startswith('ssh_host_') for f in os.listdir('/etc/ssh/'))
    daemon_running = run_command(["pgrep", "-x", "sshd"], check=False).returncode == 0
    service_enabled = run_command(["systemctl", "is-enabled", "--quiet", ssh_service], check=False).returncode == 0

    if not host_keys_exist: log("SSH host keys not found.")
    if not daemon_running: log("SSH daemon is not running.")
    if not service_enabled: log("SSH service is not enabled to start on boot.")

    if not all([host_keys_exist, daemon_running, service_enabled]):
        log("System SSH setup/activation is required...")
        if not host_keys_exist:
            log("Generating SSH host keys...")
            run_command(["ssh-keygen", "-A"], use_sudo=True)

        log(f"Enabling and starting the {ssh_service} service...")
        run_command(["systemctl", "enable", "--now", ssh_service], use_sudo=True)
        log("System SSH setup complete.")
    else:
        log("System SSH setup not needed - daemon is configured and running.")

def get_connect_address():
    """
    Determines the best address (mDNS hostname or IP) for connecting to this node.
    """
    connect_address = SLAVE_IP
    log("Checking for mDNS (Avahi) service to provide a hostname...")

    try:
        # Avahi may not be installed, so we don't treat failure as a critical error.
        if shutil.which("systemctl"):
            result = run_command(["systemctl", "is-active", "--quiet", "avahi-daemon"], check=False)
            if result.returncode == 0:
                log("Avahi daemon is active. Constructing mDNS hostname.")
                hostname = run_command(["hostname"]).stdout.strip()
                if hostname:
                    connect_address = f"{hostname}.local"
                    log(f"Node is discoverable at: {connect_address}")
            else:
                log("Avahi daemon not active or found. Node must be accessed via its IP address.")
    except Exception as e:
        err(f"A non-critical error occurred while checking for mDNS: {e}")

    return connect_address

def main():
    """Main execution function."""
    # --- PRE-FLIGHT CHECKS ---
    log("Performing pre-flight checks...")
    require_filled("MASTER_IP", MASTER_IP)
    require_filled("MASTER_SSH_PUBKEY", MASTER_SSH_PUBKEY)
    require_filled("SLAVE_IP", SLAVE_IP)

    distro_info = get_distro_info()
    if not distro_info:
        sys.exit(1)

    required_commands = [
        "sudo", "systemctl", "pgrep", "hostname", "ssh-keygen",
        distro_info['pkg_manager']
    ]
    for cmd in required_commands:
        require_cmd(cmd)
    log("Pre-flight checks passed.")

    # --- MAIN LOGIC ---
    setup_user_ssh()
    setup_system_ssh(distro_info)
    connect_address = get_connect_address()

    # --- FINAL MESSAGE ---
    log("Bootstrap complete! Slave node is ready.")
    log(f"Master can connect to this slave using: ssh {os.getlogin()}@{connect_address}")
    sys.exit(0)

if __name__ == "__main__":
    try:
        main()
    except Exception as e:
        err(f"An unexpected error occurred: {e}")
        sys.exit(1)