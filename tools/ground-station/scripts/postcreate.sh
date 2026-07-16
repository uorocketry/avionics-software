# bin/bash -i

set -e

# Get the directory of this script'
cd "$(dirname "$0")"

DIR="$(pwd)"

for script in $DIR/requirements/*.sh; do
  sudo chmod +x $script
  $script
done

echo ''
echo '🚀 You are almost ready to go!'
echo 'Re-open your terminal to update your environment'
echo ''
echo '🦀 Running `cargo build` will install and compile all rust deps'
