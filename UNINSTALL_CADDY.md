# Uninstall Caddy from Digital Ocean Droplet

This guide shows how to safely remove Caddy and migrate to the native TLS setup.

## Prerequisites

**IMPORTANT:** Before removing Caddy, make sure you have:
1. Pulled the latest code with native TLS support
2. Set up the new server with TLS certificates
3. Tested that the new setup works

## Step-by-Step Removal

### 1. Stop and Remove Caddy (Docker Version)

If you deployed Caddy via docker-compose:

```bash
# Navigate to docker directory
cd ~/rust_chat/deploy/docker

# Stop all services
docker-compose down

# Remove Caddy container and volumes
docker-compose rm -f caddy
docker volume rm docker_caddy_data docker_caddy_config

# Optional: Remove Caddy image to free space
docker rmi caddy:2-alpine
```

### 2. Stop and Remove Caddy (System Service)

If you installed Caddy as a system service:

```bash
# Stop Caddy service
sudo systemctl stop caddy

# Disable Caddy from starting on boot
sudo systemctl disable caddy

# Remove Caddy binary and service file
sudo apt remove caddy -y
# Or if installed manually:
sudo rm /usr/bin/caddy
sudo rm /etc/systemd/system/caddy.service

# Reload systemd
sudo systemctl daemon-reload
```

### 3. Remove Caddy Configuration Files

```bash
# Remove Caddy config directory
sudo rm -rf /etc/caddy

# Remove Caddy data directory (certificates stored here)
# WARNING: Only do this AFTER you've migrated certificates!
sudo rm -rf /var/lib/caddy

# Remove Caddy log directory
sudo rm -rf /var/log/caddy
```

### 4. Migrate Caddy's Let's Encrypt Certificates (Optional)

If Caddy obtained certificates for you and you want to reuse them:

```bash
# Caddy stores certs in a complex path structure
# Find your certificates:
sudo find /var/lib/caddy -name "*.crt" -o -name "*.key"

# Example path (yours may differ):
# /var/lib/caddy/.local/share/caddy/certificates/acme-v02.api.letsencrypt.org-directory/chat.yourdomain.com/

# Copy to your project (replace path with actual path from find command)
cd ~/rust_chat/deploy/docker
mkdir -p certs

# Copy certificate
sudo cp /var/lib/caddy/.local/share/caddy/certificates/acme-v02.api.letsencrypt.org-directory/chat.yourdomain.com/chat.yourdomain.com.crt certs/fullchain.pem

# Copy private key
sudo cp /var/lib/caddy/.local/share/caddy/certificates/acme-v02.api.letsencrypt.org-directory/chat.yourdomain.com/chat.yourdomain.com.key certs/privkey.pem

# Make readable
sudo chmod 644 certs/*.pem

# NOW you can safely remove /var/lib/caddy
```

**OR** Just get fresh certificates with Certbot (recommended):

```bash
# Install Certbot
sudo apt update
sudo apt install -y certbot

# Get new certificates
sudo certbot certonly --standalone \
  -d chat.yourdomain.com \
  -m your-email@example.com \
  --agree-tos \
  --non-interactive

# Copy to project
cd ~/rust_chat/deploy/docker
mkdir -p certs
sudo cp /etc/letsencrypt/live/chat.yourdomain.com/fullchain.pem certs/
sudo cp /etc/letsencrypt/live/chat.yourdomain.com/privkey.pem certs/
sudo chmod 644 certs/*.pem
```

### 5. Update Firewall

```bash
# Remove old Caddy ports
sudo ufw delete allow 80/tcp
sudo ufw delete allow 443/tcp

# Add new chat server port
sudo ufw allow 8443/tcp

# Verify
sudo ufw status
```

You should see:
```
Status: active

To                         Action      From
--                         ------      ----
22/tcp                     ALLOW       Anywhere
8443/tcp                   ALLOW       Anywhere
```

### 6. Start New Server with Native TLS

**For Docker deployment:**

```bash
cd ~/rust_chat/deploy/docker

# Pull latest code
git pull

# Rebuild with new code
docker-compose build

# Start new server with TLS
docker-compose up -d

# Check logs
docker-compose logs -f chat_server
```

You should see:
```
TLS enabled - loading certificates...
TLS certificates loaded successfully
Chat Server started at 0.0.0.0:8443
```

**For tmux deployment (Digital Ocean):**

```bash
cd ~/rust_chat/deploy/digital_ocean

# Pull latest code
cd ~/rust_chat
git pull

# Update start-server.sh with certificate paths
nano deploy/digital_ocean/start-server.sh
# Make sure TLS_CERT_PATH and TLS_KEY_PATH point to your certificates

# Start server
cd deploy/digital_ocean
./start-server.sh
```

### 7. Test the New Setup

From your local machine:

```bash
# Build client
cargo build --bin client

# Connect with TLS
./target/release/client
# Enter: tls://chat.yourdomain.com:8443
```

### 8. Clean Up Remaining Files (Optional)

```bash
# Remove Caddy user (if it exists)
sudo deluser --remove-home caddy

# Check for any remaining Caddy processes
ps aux | grep caddy

# If any found, kill them
sudo killall caddy
```

## Verification Checklist

After removal, verify everything is working:

- [ ] Caddy service is stopped: `sudo systemctl status caddy` (should show not found)
- [ ] Caddy ports removed from firewall: `sudo ufw status`
- [ ] New port 8443 is open: `sudo ufw status`
- [ ] Server is running with TLS: `docker-compose logs chat_server` or `tmux attach -t chat`
- [ ] Can connect from client: Test with `tls://your.domain.com:8443`
- [ ] Certificates are valid: `openssl s_client -connect chat.yourdomain.com:8443 -showcerts`

## Troubleshooting

### "Address already in use" on port 8443

Caddy might still be running:

```bash
# Find what's using the port
sudo netstat -tlnp | grep 8443
# or
sudo lsof -i :8443

# Kill the process
sudo kill <PID>
```

### Server can't read certificates

```bash
# Check certificates exist
ls -la ~/rust_chat/deploy/docker/certs/

# Check permissions
sudo chmod 644 ~/rust_chat/deploy/docker/certs/*.pem

# Check paths in docker-compose.yml or start-server.sh
```

### Can't get new certificates with Certbot

Port 80 needs to be free for the challenge:

```bash
# Make sure Caddy is stopped
sudo systemctl stop caddy
docker-compose stop caddy

# Check nothing is using port 80
sudo netstat -tlnp | grep :80

# Try again
sudo certbot certonly --standalone -d chat.yourdomain.com
```

### Old clients still trying to connect on port 443

Update your DNS or inform users:
- Old connection: `chat.yourdomain.com:443`
- New connection: `tls://chat.yourdomain.com:8443`

## Rollback Plan

If something goes wrong and you need Caddy back temporarily:

```bash
# Checkout previous commit
cd ~/rust_chat
git stash  # Save any local changes
git checkout <previous-commit-hash>

# Restart old setup
cd deploy/docker
docker-compose up -d

# Or for system service
sudo systemctl start caddy
```

## Complete Cleanup Command

Once you've verified everything works, run this to clean up completely:

```bash
#!/bin/bash
echo "Removing Caddy completely..."

# Stop services
sudo systemctl stop caddy 2>/dev/null || true
docker-compose stop caddy 2>/dev/null || true

# Remove packages
sudo apt remove caddy -y 2>/dev/null || true

# Remove files (CAREFUL - make sure you have certificates backed up!)
sudo rm -rf /etc/caddy
sudo rm -rf /var/lib/caddy
sudo rm -rf /var/log/caddy
sudo rm /usr/bin/caddy 2>/dev/null || true
sudo rm /etc/systemd/system/caddy.service 2>/dev/null || true

# Remove Docker volumes
docker volume rm docker_caddy_data 2>/dev/null || true
docker volume rm docker_caddy_config 2>/dev/null || true

# Remove user
sudo deluser --remove-home caddy 2>/dev/null || true

# Clean up systemd
sudo systemctl daemon-reload

echo "Caddy removal complete!"
echo "Don't forget to:"
echo "  1. Update firewall (ufw delete allow 80/tcp && ufw delete allow 443/tcp)"
echo "  2. Verify new server is running on 8443"
echo "  3. Test connection with tls:// prefix"
```

Save this as `remove-caddy.sh`, make it executable, and run:

```bash
chmod +x remove-caddy.sh
sudo ./remove-caddy.sh
```

## Summary

You've successfully:
1. ✅ Stopped Caddy service
2. ✅ Migrated or obtained new TLS certificates
3. ✅ Removed Caddy files and configuration
4. ✅ Updated firewall rules
5. ✅ Started new server with native TLS
6. ✅ Tested the connection

Your server now has native TLS support without any reverse proxy! 🎉
