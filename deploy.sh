#!/bin/bash

# Configuration
RASPBERRY_PI_IP="192.168.1.100"  # Change this to your Pi's IP address
RASPBERRY_PI_USER="pi"           # Change this to your Pi's username
REMOTE_DIR="/home/pi/vx220-dashboard"

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${GREEN}Building for Raspberry Pi...${NC}"

# Build for Raspberry Pi (ARM)
cargo build --release --target aarch64-unknown-linux-gnu

if [ $? -ne 0 ]; then
    echo -e "${RED}Build failed!${NC}"
    exit 1
fi

echo -e "${GREEN}Build successful!${NC}"
echo -e "${GREEN}Transferring files to Raspberry Pi...${NC}"

# Create remote directory if it doesn't exist
ssh ${RASPBERRY_PI_USER}@${RASPBERRY_PI_IP} "mkdir -p ${REMOTE_DIR}"

# Transfer the binary and assets
scp target/aarch64-unknown-linux-gnu/release/vx220-dashboard ${RASPBERRY_PI_USER}@${RASPBERRY_PI_IP}:${REMOTE_DIR}/
scp -r assets ${RASPBERRY_PI_USER}@${RASPBERRY_PI_IP}:${REMOTE_DIR}/

echo -e "${GREEN}Deployment complete!${NC}"
echo -e "To run the application on your Raspberry Pi, SSH into it and run:"
echo -e "cd ${REMOTE_DIR} && ./vx220-dashboard" 