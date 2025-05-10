# Configuration
$RASPBERRY_PI_IP = "192.168.1.24"  # Your Pi's IP address
$RASPBERRY_PI_USER = "vx220turbo"  # Your Pi's username
$REMOTE_DIR = "/home/vx220turbo/vx220-dashboard"

Write-Host "Preparing deployment..." -ForegroundColor Green

# Create remote directory if it doesn't exist
ssh "${RASPBERRY_PI_USER}@${RASPBERRY_PI_IP}" "mkdir -p ${REMOTE_DIR}"

# Install required dependencies on Raspberry Pi
Write-Host "Installing dependencies on Raspberry Pi..." -ForegroundColor Green
ssh "${RASPBERRY_PI_USER}@${RASPBERRY_PI_IP}" "sudo apt-get update && sudo apt-get install -y build-essential pkg-config libdbus-1-dev libgl1-mesa-dev libx11-dev libxrandr-dev libxinerama-dev libxcursor-dev libxi-dev"

# Copy the source code to Raspberry Pi
Write-Host "Copying source code to Raspberry Pi..." -ForegroundColor Green
scp -r "src" "${RASPBERRY_PI_USER}@${RASPBERRY_PI_IP}:${REMOTE_DIR}/"
scp -r "assets" "${RASPBERRY_PI_USER}@${RASPBERRY_PI_IP}:${REMOTE_DIR}/"
scp "Cargo.toml" "${RASPBERRY_PI_USER}@${RASPBERRY_PI_IP}:${REMOTE_DIR}/"
scp "Cargo.lock" "${RASPBERRY_PI_USER}@${RASPBERRY_PI_IP}:${REMOTE_DIR}/"

# Build on Raspberry Pi
Write-Host "Building on Raspberry Pi..." -ForegroundColor Green
ssh "${RASPBERRY_PI_USER}@${RASPBERRY_PI_IP}" 'source $HOME/.cargo/env && cd '"${REMOTE_DIR}"' && cargo build'

Write-Host "Deployment complete!" -ForegroundColor Green
Write-Host "To run the application on your Raspberry Pi, SSH into it and run:"
Write-Host "cd ${REMOTE_DIR} && ./target/release/vx220-dashboard" 