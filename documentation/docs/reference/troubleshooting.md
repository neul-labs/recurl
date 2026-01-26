# Troubleshooting

Solutions to common issues with recurl.

---

## Installation Issues

### "Command not found: recurl"

The binary is not in your PATH.

**Solution:**

```bash
# Add to PATH (Linux/macOS)
export PATH="/usr/local/recurl:$PATH"

# Or use full path
/usr/local/recurl/recurl --version
```

### Alias Not Working

The shell alias may not be set up correctly.

**Solution:**

```bash
# Check if alias exists
alias curl

# Re-add alias
echo 'alias curl="/usr/local/recurl/recurl"' >> ~/.bashrc
source ~/.bashrc
```

---

## Chromium Issues

### "Chromium auto-download not available"

You're on Linux ARM64 where auto-download isn't supported.

**Solution:**

```bash
# Install Chromium manually
# Ubuntu/Debian:
sudo apt install chromium-browser

# Fedora:
sudo dnf install chromium

# Arch:
sudo pacman -S chromium
```

### "Failed to launch browser"

Chromium couldn't start. Common causes:

1. **Missing dependencies**
   ```bash
   # Install dependencies (Debian/Ubuntu)
   sudo apt install -y \
       libnss3 libatk-bridge2.0-0 libdrm2 libxkbcommon0 \
       libxcomposite1 libxdamage1 libxrandr2 libgbm1 libasound2
   ```

2. **No display (headless server)**
   ```bash
   # Ensure using headless mode (default)
   # If issues persist, try:
   export DISPLAY=:0
   ```

3. **Sandbox issues**
   ```bash
   # recurl already uses --no-sandbox
   # If still failing, check permissions
   ```

### "Chromium download timeout"

Network issues during Chromium download.

**Solution:**

```bash
# Retry with debug
recurl --recurl-debug --recurl-js https://example.com

# Or use system Chrome
# Install Chrome and recurl will find it
```

---

## Request Issues

### Still Getting 403 After Escalation

The site may have additional protections.

**Try:**

1. **Force JS preflight**
   ```bash
   recurl --recurl-js https://site.com
   ```

2. **Wait for specific element**
   ```bash
   recurl --recurl-js --recurl-js-wait ".content-loaded" https://site.com
   ```

3. **Increase timeout**
   ```bash
   recurl --recurl-js --recurl-js-timeout 60000 https://site.com
   ```

4. **Check debug output**
   ```bash
   recurl --recurl-debug --recurl-js https://site.com
   ```

### Request Hangs

Possible timeout or infinite wait.

**Solution:**

```bash
# Set explicit timeout
recurl --recurl-js --recurl-js-timeout 30000 https://site.com

# Or use curl timeout
recurl --max-time 60 https://site.com
```

### Wrong Content Returned

The page may require JavaScript rendering.

**Solution:**

```bash
# Get rendered HTML
recurl --recurl-js-rendered https://spa-site.com

# Wait for content
recurl --recurl-js-rendered --recurl-js-wait "#app-loaded" https://spa-site.com
```

---

## Daemon Issues

### "Failed to connect to daemon"

The daemon may not be running or socket is stale.

**Solution:**

```bash
# Remove stale socket (Linux/macOS)
rm /tmp/recurl.*.sock

# Try again
recurl --recurl-js https://example.com
```

### Daemon Using Too Much Memory

The browser pool consumes memory.

**Solution:**

```bash
# Reduce idle timeout
export RCURL_DAEMON_IDLE_MS=10000

# Or disable daemon
recurl --recurl-daemon off --recurl-js https://example.com
```

### Daemon Won't Stop

Force kill the daemon process.

**Solution:**

```bash
# Find and kill (Linux/macOS)
pkill -f recurld

# Windows
taskkill /IM recurld.exe /F
```

---

## Performance Issues

### Slow First Request

First JS preflight downloads Chromium and launches browser.

**This is normal.** Subsequent requests will be faster.

### Slow Subsequent Requests

Daemon may not be running.

**Check:**

```bash
# See if daemon is active
ps aux | grep recurld

# Enable debug to see daemon usage
recurl --recurl-debug --recurl-js https://example.com
```

### High CPU Usage

Browser automation is CPU-intensive.

**Mitigations:**

```bash
# Reduce daemon timeout
export RCURL_DAEMON_IDLE_MS=30000

# Disable daemon for one-off requests
recurl --recurl-daemon off --recurl-js https://example.com
```

---

## Debug Mode

Always start troubleshooting with debug mode:

```bash
recurl --recurl-debug https://problematic-site.com
```

**Debug output shows:**

- Which engine was used
- Detection results
- Escalation steps
- Cookie extraction
- Final result

### Combined with curl verbose

```bash
recurl --recurl-debug -v https://problematic-site.com
```

Shows both recurl decisions and curl network details.

---

## Common Error Messages

| Error | Meaning | Solution |
|-------|---------|----------|
| `curl_engine: 403` | Request blocked | Let recurl escalate or use `--recurl-js` |
| `Detected: Cloudflare` | CF protection found | Normal, recurl will escalate |
| `Browser launch timeout` | Chromium took too long | Check system resources |
| `Navigation timeout` | Page load timeout | Increase `--recurl-js-timeout` |
| `Selector not found` | Wait element missing | Check selector or increase timeout |
| `Failed to create socket` | IPC issue | Remove stale socket file |

---

## Getting Help

### Provide Debug Output

When reporting issues, include:

```bash
recurl --recurl-debug --recurl-js https://problematic-site.com 2>&1
```

### Check Version

```bash
recurl --version
```

### System Information

```bash
uname -a
which chromium google-chrome
```

### Report Issues

Open an issue at [GitHub Issues](https://github.com/user/recurl/issues) with:

1. recurl version
2. Operating system and architecture
3. Debug output
4. Steps to reproduce
