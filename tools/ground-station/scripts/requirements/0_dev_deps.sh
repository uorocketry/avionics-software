# bin/bash -i

echo '📥 Updating system'
sudo apt -qq update && sudo apt -qq upgrade -y
echo '✅ System updated'

echo '📥 Installing dev dependencies'
sudo apt -qq install -y \
    pkg-config \
    libudev-dev \
    inotify-tools \
    cmake \
    libusb-1.0-0-dev
echo '✅ Dev dependencies installed'
