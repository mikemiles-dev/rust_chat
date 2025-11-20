# Ban Strategy for Online Hosting

## Overview

When hosting a chat server publicly, you need multiple layers of protection against abuse.

## Multi-Layer Ban Strategy

### Layer 1: Rate Limiting (Already Implemented ✅)

**Current Implementation:**
- 10 messages per second per connection
- Token bucket algorithm with auto-refill

**Why it works:**
- Prevents spam floods
- Stops basic bot attacks
- No configuration needed

### Layer 2: IP-Based Banning (Recommended)

**Implementation Options:**

#### Option A: Application-Level IP Bans

Add IP tracking and blocking in the Rust server:

```rust
// In server/src/main.rs
use std::collections::HashSet;

pub struct ChatServer {
    // ... existing fields
    banned_ips: Arc<RwLock<HashSet<IpAddr>>>,
}

// Before accepting connection:
if self.banned_ips.read().await.contains(&addr.ip()) {
    logger::log_warning(&format!("Rejected banned IP: {}", addr.ip()));
    continue;
}
```

**Pros:**
- Fast blocking before any processing
- Programmatic control
- Can persist to file/database

**Cons:**
- Requires code changes
- VPN users can bypass by changing IPs

#### Option B: Firewall-Level Bans (Recommended)

Use `iptables` or `ufw` to block IPs at the firewall:

```bash
# Block single IP
sudo ufw deny from 1.2.3.4

# Block IP range
sudo ufw deny from 1.2.3.0/24

# List banned IPs
sudo ufw status numbered

# Remove ban
sudo ufw delete [number]
```

**Pros:**
- Blocks before TCP connection established
- No server resource usage for banned IPs
- Works for all services, not just chat
- Easy to script

**Cons:**
- Manual process
- Need server access

### Layer 3: Username Banning

**Implementation:**

```rust
// Track banned usernames
pub struct ChatServer {
    banned_usernames: Arc<RwLock<HashSet<String>>>,
}

// In user_connection/handlers.rs - during Join:
if self.banned_usernames.read().await.contains(&username) {
    let error_msg = ChatMessage::try_new(
        MessageTypes::Error,
        Some("This username is banned".as_bytes().to_vec())
    )?;
    self.send_message_chunked(error_msg).await?;
    return Err(UserConnectionError::Banned);
}
```

**Pros:**
- Prevents specific troublemakers
- Easy to implement
- Can be bypassed but annoying

**Cons:**
- Easy to bypass (change username)
- Only useful for persistent trolls

### Layer 4: Pattern-Based Detection

**Detect and auto-ban based on behavior:**

```rust
// Track user behavior
struct UserBehavior {
    message_count: usize,
    spam_score: f32,
    violations: usize,
}

// Auto-ban triggers:
// - Repeated identical messages
// - All caps messages
// - URL spam
// - Profanity (if you want to filter)
// - Extremely rapid username changes
```

**Example - Duplicate Message Detection:**

```rust
struct UserConnection {
    last_messages: VecDeque<String>, // Keep last 5 messages
}

// In message handler:
if self.last_messages.iter().filter(|m| **m == content).count() >= 3 {
    // Same message 3+ times = spam
    self.auto_ban("Spam detected").await?;
}
```

### Layer 5: Cloudflare (Best for DDoS)

**Use Cloudflare in front of Caddy:**

```
Internet → Cloudflare → Your Server (Caddy → Chat)
```

**Benefits:**
- DDoS protection
- IP blocking at edge
- Rate limiting
- Bot detection
- Free tier available

**Setup:**
1. Add domain to Cloudflare
2. Point DNS to Cloudflare nameservers
3. Enable "Proxy" mode
4. Configure firewall rules

**Cloudflare Firewall Rules (Free):**
```
# Block specific countries
Country not in {US, CA, UK, ...}

# Block known bad ASNs
AS Number in {list of VPN/hosting providers}

# Rate limit per IP
Rate limit: 100 requests per minute

# Challenge suspicious traffic
Threat Score > 10 → Challenge
```

### Layer 6: Account System (Advanced)

**Require registration before chatting:**

- Email verification
- CAPTCHA on signup
- Rate limit account creation per IP
- Require established account age before chatting

**Pros:**
- Very effective against bots
- Creates accountability
- Can ban accounts permanently

**Cons:**
- Reduces casual usage
- More complex to implement
- Privacy concerns

## Recommended Setup for Small/Medium Server

### Immediate (No code changes)

1. **Caddy Rate Limiting** (already configured in Caddyfile)
2. **UFW Firewall** for manual IP bans
3. **Cloudflare Free** for DDoS protection

### Short-term (Minimal code changes)

4. **IP Ban List** - Simple text file loaded on startup
5. **Username Ban List** - Prevent known troublemakers
6. **Admin Commands** - `/ban <username>` and `/banip <ip>`

### Long-term (If abuse becomes serious)

7. **Auto-ban System** - Pattern detection
8. **Account System** - Email verification
9. **Logging System** - Track all violations
10. **Monitoring** - Alert on suspicious patterns

## Implementation Priority

### Phase 1: Infrastructure (Do First)
```bash
# Enable Cloudflare
# - Sign up at cloudflare.com
# - Add your domain
# - Enable proxy mode

# Configure firewall
sudo ufw enable
sudo ufw allow 22,80,443/tcp

# Create ban script
cat > /usr/local/bin/ban-ip << 'EOF'
#!/bin/bash
sudo ufw deny from "$1"
echo "Banned IP: $1"
EOF
chmod +x /usr/local/bin/ban-ip
```

### Phase 2: Application Bans (Quick Wins)

Add these files:

**banned_ips.txt:**
```
1.2.3.4
5.6.7.8
192.168.1.0/24
```

**banned_usernames.txt:**
```
admin
moderator
spammer123
offensive_name
```

Load on startup and check before accepting connections.

### Phase 3: Admin Commands

Add these server commands:
- `/ban <username>` - Ban by username
- `/banip <ip>` - Ban by IP
- `/unban <username>` - Unban username
- `/unbanip <ip>` - Unban IP
- `/banlist` - Show all bans

## Detection Patterns

### Spam Indicators
- Same message 3+ times in 30 seconds
- Messages > 5 per second for 10+ seconds
- URLs in >50% of messages
- All caps for 5+ consecutive messages

### Bot Indicators
- Connect and disconnect rapidly (>10 times/minute)
- Identical message patterns
- No human-like delays between messages
- Sequential username patterns (bot1, bot2, bot3)

### Attack Indicators
- Many connections from same IP
- Connections from known VPN/proxy ASNs
- Geographic anomalies (all from one country suddenly)
- Port scanning (multiple ports tried)

## Monitoring Commands

```bash
# Watch active connections
watch -n 1 'ss -tn | grep :8080 | wc -l'

# Monitor logs for suspicious patterns
sudo journalctl -u rust-chat -f | grep -i "spam\|ban\|kick"

# Check connection sources
sudo netstat -tn | grep :8080 | awk '{print $5}' | cut -d: -f1 | sort | uniq -c | sort -rn

# Alert on high connection count
while true; do
  COUNT=$(ss -tn | grep :8080 | wc -l)
  if [ $COUNT -gt 50 ]; then
    echo "ALERT: $COUNT connections detected!"
  fi
  sleep 5
done
```

## Example: Quick Ban Script

```bash
#!/bin/bash
# /usr/local/bin/chat-ban

ACTION=$1
TARGET=$2

case $ACTION in
  ip)
    sudo ufw deny from "$TARGET"
    echo "$TARGET" >> /opt/rust_chat/banned_ips.txt
    echo "Banned IP: $TARGET"
    ;;
  username)
    echo "$TARGET" >> /opt/rust_chat/banned_usernames.txt
    # Send kick command via docker exec or systemctl
    echo "Banned username: $TARGET"
    ;;
  list)
    echo "=== Banned IPs ==="
    cat /opt/rust_chat/banned_ips.txt 2>/dev/null
    echo ""
    echo "=== Banned Usernames ==="
    cat /opt/rust_chat/banned_usernames.txt 2>/dev/null
    ;;
  *)
    echo "Usage: chat-ban {ip|username|list} <target>"
    ;;
esac
```

## Cost-Effective Strategy

**Free Tier Setup:**
- ✅ Cloudflare Free (DDoS + basic firewall)
- ✅ UFW (included with Ubuntu)
- ✅ Application rate limiting (already implemented)
- ✅ Manual IP bans via firewall

**Low-Cost Additions:**
- Cloudflare Pro ($20/month) - Advanced DDoS, more firewall rules
- Fail2ban - Auto-ban on patterns
- Simple logging to file

## What NOT to Do

❌ **Don't implement captcha on every message** - Ruins UX
❌ **Don't ban entire countries** - Too broad, false positives
❌ **Don't trust IP bans alone** - VPNs make them ineffective
❌ **Don't auto-ban without logging** - Need evidence for appeals
❌ **Don't ban on first violation** - Could be accident
❌ **Don't store IPs without privacy policy** - Legal requirements

## Privacy Considerations

**If you log IPs or ban users:**
1. Post a privacy policy
2. Comply with GDPR (if EU users)
3. Provide appeal process
4. Auto-delete old logs (30-90 days)
5. Don't share user data

**Minimal Logging Approach:**
```
# Log bans but not regular traffic
✅ Ban events: IP, reason, timestamp
✅ Abuse attempts: Pattern, count
❌ Regular chat messages
❌ User IPs in normal logs
```

## Testing Your Defenses

```bash
# Test rate limiting (should get kicked)
for i in {1..100}; do
  echo "spam $i" | nc yourserver.com 8080 &
done

# Test multiple connections (should be limited)
for i in {1..150}; do
  nc yourserver.com 8080 &
done

# Test banned IP (should be rejected)
sudo ufw deny from 127.0.0.1
nc localhost 8080  # Should fail
```

## Recommended Final Setup

For a public chat server, use this stack:

```
Internet
   ↓
Cloudflare (Free)
   ↓ (DDoS protection, rate limiting, bot filtering)
Your Server
   ↓
Caddy (HTTPS, reverse proxy)
   ↓
Rust Chat Server
   - Built-in rate limiting (10/sec)
   - IP ban list (loaded from file)
   - Username ban list
   - Manual /kick command

Firewall (UFW)
   - Block specific IPs
   - Default deny except 22,80,443
```

**Daily Maintenance:**
- Check logs for abuse patterns
- Review ban list (unban false positives)
- Update banned_ips.txt and banned_usernames.txt
- Monitor connection counts

**Monthly:**
- Review Cloudflare analytics
- Update dependencies (security patches)
- Rotate logs

This gives you protection against:
- ✅ Spam (rate limiting)
- ✅ DDoS (Cloudflare)
- ✅ Bots (rate limiting + Cloudflare)
- ✅ Trolls (kick + IP ban)
- ✅ Abuse (pattern detection)

All without requiring user accounts or complex systems!
