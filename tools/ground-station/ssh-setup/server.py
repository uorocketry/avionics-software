#!/usr/bin/env python3

import os
import sys
import re
import shutil
import socket
import subprocess
from http.server import BaseHTTPRequestHandler, HTTPServer

# --- UTILITY FUNCTIONS ---

def get_primary_ip():
    """
    Determines the primary IP for an offline LAN, ignoring virtual interfaces.
    It tries the 'ip' command first, then falls back to 'ifconfig'.
    """
    ignored_prefixes = ('lo', 'docker', 'virbr', 'veth', 'br-')

    # Method 1: Use the 'ip' command (Modern Linux)
    if shutil.which("ip"):
        try:
            result = subprocess.run(["ip", "-o", "link", "show"], capture_output=True, text=True, check=True)
            for line in result.stdout.strip().split('\n'):
                match = re.search(r'^\d+:\s+([^:]+):', line)
                if not match: continue

                iface_name = match.group(1).strip()
                if any(iface_name.startswith(p) for p in ignored_prefixes): continue

                ip_result = subprocess.run(["ip", "-4", "addr", "show", "dev", iface_name], capture_output=True, text=True)
                ip_match = re.search(r"inet\s+([\d.]+)/", ip_result.stdout)
                if ip_match:
                    ip = ip_match.group(1)
                    print(f"[INFO] Found IP '{ip}' on interface '{iface_name}' via 'ip' command.")
                    return ip
        except Exception as e:
            print(f"[WARNING] 'ip' command method failed: {e}")

    # Method 2: Fallback to 'ifconfig' (Older systems)
    if shutil.which("ifconfig"):
        try:
            result = subprocess.run(["ifconfig"], capture_output=True, text=True, check=True)
            for block in result.stdout.strip().split('\n\n'):
                iface_name = block.split()[0].rstrip(':')
                if any(iface_name.startswith(p) for p in ignored_prefixes): continue

                ip_match = re.search(r"inet(?: addr:)?\s*([\d.]+)", block)
                if ip_match and not ip_match.group(1).startswith("127."):
                    ip = ip_match.group(1)
                    print(f"[INFO] Found IP '{ip}' on interface '{iface_name}' via 'ifconfig'.")
                    return ip
        except Exception as e:
            print(f"[WARNING] 'ifconfig' command method failed: {e}")

    return None

# --- CONFIGURATION ---

# 1. Determine Master IP
MASTER_IP_VAL = get_primary_ip()
if not MASTER_IP_VAL:
    print("[CRITICAL] Could not auto-detect a primary IP. Aborting.")
    sys.exit(1)

# 2. Load or Generate Master SSH Key from the standard user location
ssh_dir = os.path.expanduser("~/.ssh")
key_path = os.path.join(ssh_dir, "id_rsa")
pub_key_path = f"{key_path}.pub"

if os.path.exists(pub_key_path):
    print(f"[INFO] Using existing SSH key: {pub_key_path}")
    with open(pub_key_path, "r") as f:
        MASTER_SSH_PUBKEY_VAL = f.read().strip()
else:
    print(f"[WARNING] SSH key '{pub_key_path}' not found. Generating a new one...")
    if not shutil.which("ssh-keygen"):
        print("[CRITICAL] 'ssh-keygen' command not found. Cannot generate key. Aborting.")
        sys.exit(1)

    try:
        # Ensure the .ssh directory exists with correct permissions (drwx------)
        os.makedirs(ssh_dir, mode=0o700, exist_ok=True)

        hostname = socket.gethostname()
        keygen_command = [
            "ssh-keygen", "-t", "rsa", "-b", "4096", # Strong RSA key
            "-f", key_path,                         # Output file path (~/.ssh/id_rsa)
            "-N", "",                               # Empty passphrase
        ]
        subprocess.run(keygen_command, check=True, capture_output=True, text=True)
        print(f"[INFO] Successfully generated new SSH key pair in {ssh_dir}")
        with open(pub_key_path, "r") as f:
            MASTER_SSH_PUBKEY_VAL = f.read().strip()
    except subprocess.CalledProcessError as e:
        print("[CRITICAL] Failed to generate SSH key.")
        print(f"STDERR: {e.stderr.strip()}")
        sys.exit(1)

SERVER_HOST = "0.0.0.0"
SERVER_PORT = 8000
TEMPLATE_FILE = "bootstrap.py"

# --- HTTP SERVER LOGIC ---

class TemplatingRequestHandler(BaseHTTPRequestHandler):
    def do_GET(self):
        if self.path == "/install":
            self.serve_template()
        else:
            self.send_error(404, "Not Found")
            self.end_headers()
            self.wfile.write(b"Error: Endpoint not found. Use /install\n")

    def serve_template(self):
        try:
            slave_ip = self.client_address[0]
            print(f"[INFO] Request for /install from slave at {slave_ip}")

            with open(TEMPLATE_FILE, "r") as f:
                template_content = f.read()

            rendered_script = template_content.replace("{{MASTER_IP}}", MASTER_IP_VAL)
            rendered_script = rendered_script.replace("{{MASTER_SSH_PUBKEY}}", MASTER_SSH_PUBKEY_VAL)
            rendered_script = rendered_script.replace("{{SLAVE_IP}}", slave_ip)

            self.send_response(200)
            self.send_header("Content-type", "text/plain; charset=utf-8")
            self.end_headers()
            self.wfile.write(rendered_script.encode("utf-8"))
        except FileNotFoundError:
            print(f"[ERROR] Template file '{TEMPLATE_FILE}' not found!")
            self.send_error(500, "Internal Server Error")
        except Exception as e:
            print(f"[ERROR] An unexpected error occurred: {e}")
            self.send_error(500, "Internal Server Error")

def run_server():
    """Starts the HTTP server."""
    try:
        server_address = (SERVER_HOST, SERVER_PORT)
        httpd = HTTPServer(server_address, TemplatingRequestHandler)
        print("\n--- Bootstrap Server Running ---")
        print(f"Auto-detected Master IP: {MASTER_IP_VAL}")
        print(f"Server listening on http://{SERVER_HOST}:{SERVER_PORT}")
        print("\nTo install on a slave node, run:")
        print(f"\033[0;32mcurl -sSL http://{MASTER_IP_VAL}:{SERVER_PORT}/install | python3\033[0m")
        print("\n---------------------------------")
        print("Press Ctrl+C to stop the server.")
        httpd.serve_forever()
    except KeyboardInterrupt:
        print("\n[INFO] Server is shutting down.")
        httpd.server_close()
    except OSError as e:
        print(f"[ERROR] Could not start server: {e}")

if __name__ == "__main__":
    if not os.path.exists(TEMPLATE_FILE):
        print(f"[ERROR] Template file './{TEMPLATE_FILE}' not found. Aborting.")
    else:
        run_server()