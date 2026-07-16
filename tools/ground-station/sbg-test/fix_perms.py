#!/usr/bin/env python3
"""
USB/Serial Terminal Permissions Fixer

Detects serial device files under /dev and adds the current user
to the owning groups (e.g., uucp, dialout) for serial/USB serial access.
"""

import os
import sys
import subprocess
import argparse
import glob
import grp

def get_serial_groups() -> list[str]:
    """
    Inspect common serial device file patterns and return the set of
    non-root groups owning those devices.
    """
    patterns = ['/dev/ttyS*', '/dev/ttyUSB*', '/dev/ttyACM*']
    groups = set()
    for pat in patterns:
        for dev in glob.glob(pat):
            try:
                st = os.stat(dev)
                group = grp.getgrgid(st.st_gid).gr_name
                if group != 'root':
                    groups.add(group)
            except FileNotFoundError:
                continue
    return sorted(groups)

def run_command(cmd: list[str], dry_run: bool) -> int:
    """Run a system command, optionally in dry-run mode."""
    if dry_run:
        print(f"DRY RUN: {' '.join(cmd)}")
        return 0
    return subprocess.run(cmd, check=False).returncode

def add_user_to_group(user: str, group: str, dry_run: bool=False) -> bool:
    """
    Add the specified user to the given group.
    """
    # Check current groups
    out = subprocess.run(['groups', user], capture_output=True, text=True)
    if group in out.stdout.split():
        print(f"User {user} already in group {group}")
        return True
    # Add to group
    cmd = ['sudo', 'usermod', '-aG', group, user]
    rc = run_command(cmd, dry_run)
    if rc == 0:
        print(f"Added {user} to {group}")
        return True
    else:
        print(f"Failed to add {user} to {group}", file=sys.stderr)
        return False

def main():
    parser = argparse.ArgumentParser(
        description="Fix permissions for USB/serial devices"
    )
    parser.add_argument('--dry-run', action='store_true',
                        help='Show what would be done without making changes')
    args = parser.parse_args()

    current_user = os.getenv('USER') or os.getenv('LOGNAME') or 'unknown'
    groups = get_serial_groups()

    if not groups:
        print("No serial devices detected.")
        sys.exit(1)

    print(f"Detected serial device groups: {', '.join(groups)}")
    success = True
    for g in groups:
        if not add_user_to_group(current_user, g, args.dry_run):
            success = False

    if not args.dry_run:
        print("→ Remember to reboot for changes to take effect.")

    sys.exit(0 if success else 1)

if __name__ == "__main__":
    main()
