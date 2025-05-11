# Configuration
$RASPBERRY_PI_IP = "192.168.1.24"  # Change to your Pi's IP
$RASPBERRY_PI_USER = "vx220turbo"  # Change to your Pi's username
$REMOTE_DIR = "/home/vx220turbo"
$SSH_KEY_PATH = "$env:USERPROFILE\.ssh\id_ed25519"

# 1. Generate SSH key if it doesn't exist
if (!(Test-Path $SSH_KEY_PATH)) {
    Write-Host "No SSH key found. Generating a new SSH key..." -ForegroundColor Yellow
    ssh-keygen -t ed25519 -f $SSH_KEY_PATH -N "" -C "vx220-windows-auto-key"
} else {
    Write-Host "SSH key already exists at $SSH_KEY_PATH" -ForegroundColor Green
}

# 2. Copy public key to Raspberry Pi (manual method for Windows, using Get-Content)
Write-Host "Copying SSH public key to Raspberry Pi for passwordless authentication..." -ForegroundColor Green
$pubKey = Get-Content "$SSH_KEY_PATH.pub" -Raw
ssh $RASPBERRY_PI_USER@$RASPBERRY_PI_IP "mkdir -p ~/.ssh && echo '$pubKey' >> ~/.ssh/authorized_keys"

Write-Host "Testing passwordless SSH login..." -ForegroundColor Green
$sshTest = ssh -o BatchMode=yes -o ConnectTimeout=5 "$RASPBERRY_PI_USER@$RASPBERRY_PI_IP" "echo success" 2>$null
if ($sshTest -ne "success") {
    Write-Host "Passwordless SSH setup failed. Please check your credentials and try again." -ForegroundColor Red
    exit 1
}

# 3. Copy raspi_setup.sh and execute it
Write-Host "Copying raspi_setup.sh to Raspberry Pi..." -ForegroundColor Green
scp raspi_setup.sh "${RASPBERRY_PI_USER}@${RASPBERRY_PI_IP}:${REMOTE_DIR}/"

Write-Host "Running raspi_setup.sh on Raspberry Pi..." -ForegroundColor Green
ssh "${RASPBERRY_PI_USER}@${RASPBERRY_PI_IP}" "chmod +x ${REMOTE_DIR}/raspi_setup.sh && ${REMOTE_DIR}/raspi_setup.sh"

# 4. Setup the Serial Connection to the ESP32
# Add user to dialout group for serial port access
Write-Host "Adding user to dialout group for serial port access..." -ForegroundColor Green
ssh "${RASPBERRY_PI_USER}@${RASPBERRY_PI_IP}" "sudo chgrp dialout /dev/ttyS0 && sudo chmod 660 /dev/ttyS0"
ssh "${RASPBERRY_PI_USER}@${RASPBERRY_PI_IP}" "sudo usermod -a -G dialout ${RASPBERRY_PI_USER}"


Write-Host "Setup script executed on Raspberry Pi." -ForegroundColor Green 