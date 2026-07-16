

# sudo date -s "2025-08-05 15:30:00"
# sudo timedatectl set-ntp true

sudo systemctl restart systemd-timesyncd


sudo apt update

sudo apt upgrade

sudo apt install -y git curl

# sbg e com compilation
sudo apt install -y cmake build-essential

curl -LsSf https://astral.sh/uv/install.sh | sh

source $HOME/.local/bin/env

# set up odrive usb permissions

sudo tee /etc/udev/rules.d/91-odrive.rules > /dev/null <<'EOF'
SUBSYSTEM=="usb", ATTR{idVendor}=="1209", ATTR{idProduct}=="0d3[0-9]", MODE="0666", ENV{ID_MM_DEVICE_IGNORE}="1"
SUBSYSTEM=="usb", ATTR{idVendor}=="0483", ATTR{idProduct}=="df11", MODE="0666"
EOF


sudo tee /etc/udev/rules.d/99-sbg-ellipse.rules > /dev/null <<EOF
SUBSYSTEM=="tty", ATTRS{idVendor}=="0403", ATTRS{idProduct}=="6001", SYMLINK+="ellipse"
EOF


sudo udevadm control --reload-rules
sudo udevadm trigger
