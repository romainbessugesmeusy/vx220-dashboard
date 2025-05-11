#!/bin/bash

set -e

# Update package list
sudo apt-get update

# Install build essentials and dependencies
sudo apt-get install -y \
    build-essential \
    pkg-config \
    libdbus-1-dev \
    libgl1-mesa-dev \
    libx11-dev \
    libxrandr-dev \
    libxinerama-dev \
    libxcursor-dev \
    libxi-dev \
    git \
    curl

# Install Rust (if not already installed)
if ! command -v cargo &> /dev/null; then
    echo "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source $HOME/.cargo/env
else
    echo "Rust is already installed."
fi

# Add Rust to PATH for future sessions
if ! grep -q 'cargo/env' ~/.bashrc; then
    echo 'source $HOME/.cargo/env' >> ~/.bashrc
fi

# Enable UART in config.txt
echo "Configuring UART..."
if ! grep -q "^enable_uart=1" /boot/firmware/config.txt; then
    echo "Adding enable_uart=1 to /boot/firmware/config.txt"
    sudo sh -c 'echo "enable_uart=1" >> /boot/firmware/config.txt'
fi


echo "Setup complete! You may need to restart your terminal or run 'source ~/.cargo/env' to use Rust." 