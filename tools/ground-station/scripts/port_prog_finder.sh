#!/bin/sh

if [ -z "$1" ]; then
  echo "Usage: $0 <port>"
  exit 1
fi

# sudo lsof -n -i :5656 | grep LISTEN
sudo lsof -n -i :$1 | grep LISTEN