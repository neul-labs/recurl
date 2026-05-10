# Installation

Detailed installation instructions for all platforms.

---

## Quick Install

### Linux / macOS

```bash
curl -fsSL https://recurl.dev/install.sh | bash
```

With options:

```bash
# Specific version
RCURL_VERSION=v0.1.2 curl -fsSL https://recurl.dev/install.sh | bash

# Custom install directory
RCURL_INSTALL_DIR=/opt/recurl curl -fsSL https://recurl.dev/install.sh | bash
```

### Windows (PowerShell)

```powershell
irm https://recurl.dev/install.ps1 | iex
```

With options:

```powershell
# Specific version
$env:RCURL_VERSION = "v0.1.2"; irm https://recurl.dev/install.ps1 | iex
```

---

## Package Managers

### Homebrew (macOS / Linux)

```bash
# Add the tap
brew tap neul-labs/tap

# Install
brew install recurl
```

Or install directly:

```bash
brew install neul-labs/tap/recurl
```

### npm (Node.js)

```bash
# Global install
npm install -g recurl-cli

# Or run without installing
npx recurl-cli https://example.com
```

### PyPI (Python)

```bash
# Install
pip install recurl-cli

# Run
python -m recurl https://example.com
```

### Scoop (Windows)

```powershell
# Add the bucket
scoop bucket add recurl https://github.com/neul-labs/recurl

# Install
scoop install recurl
```

### AUR (Arch Linux)

```bash
# Using yay
yay -S recurl

# Or using paru
paru -S recurl
```

---

## Docker

### Quick Run

```bash
# Single request
docker run --rm ghcr.io/neul-labs/recurl https://example.com

# Save output to local directory
docker run --rm -v $(pwd)/output:/output ghcr.io/neul-labs/recurl \
    https://example.com -o /output/result.html
```

### Docker Compose

```yaml
# docker-compose.yml
version: '3.8'
services:
  recurl:
    image: ghcr.io/neul-labs/recurl
    volumes:
      - ./output:/output
```

```bash
docker-compose run --rm recurl https://example.com
```

---

## Manual Installation

### Download

Download from [GitHub Releases](https://github.com/neul-labs/recurl/releases):

| Platform | Archive |
|----------|---------|
| Linux x86_64 | `recurl-linux-x86_64.tar.gz` |
| Linux aarch64 | `recurl-linux-aarch64.tar.gz` |
| macOS x86_64 | `recurl-darwin-x86_64.tar.gz` |
| macOS aarch64 | `recurl-darwin-aarch64.tar.gz` |
| Windows x86_64 | `recurl-windows-x86_64.zip` |

### Directory Structure

After extraction:

```
recurl/
├── recurl(.exe)              # Main binary
├── recurld(.exe)             # Daemon binary
└── bin/
    ├── curl_engine(.exe)    # Upstream curl
    ├── curl_chrome          # Chrome impersonation (Linux/macOS)
    ├── curl_ff              # Firefox impersonation (Linux/macOS)
    └── curl_safari          # Safari impersonation (Linux/macOS)
```

### Linux (Manual)

```bash
# Extract
tar -xzf recurl-linux-x86_64.tar.gz
sudo mv recurl /usr/local/

# Add to PATH
echo 'export PATH="/usr/local/recurl:$PATH"' >> ~/.bashrc
source ~/.bashrc

# Optional: alias curl
echo 'alias curl="/usr/local/recurl/recurl"' >> ~/.bashrc
source ~/.bashrc
```

### macOS (Manual)

```bash
# Extract
tar -xzf recurl-darwin-aarch64.tar.gz
sudo mv recurl /usr/local/

# Add alias
echo 'alias curl="/usr/local/recurl/recurl"' >> ~/.zshrc
source ~/.zshrc
```

### Windows (Manual)

1. Extract `recurl-windows-x86_64.zip` to `C:\Program Files\recurl\`

2. Add to PATH:
    - Open System Properties → Environment Variables
    - Edit `Path` under User variables
    - Add `C:\Program Files\recurl`

3. Set up alias in PowerShell profile (`$PROFILE`):

    ```powershell
    Set-Alias -Name curl -Value "C:\Program Files\recurl\recurl.exe" -Option AllScope
    ```

---

## Building from Source

### Prerequisites

- Rust 1.75 or later
- pkg-config and OpenSSL headers (Linux)

### Build

=== "Linux"

    ```bash
    # Install dependencies (Debian/Ubuntu)
    sudo apt-get update
    sudo apt-get install -y build-essential pkg-config libssl-dev

    # Clone and build
    git clone https://github.com/neul-labs/recurl.git
    cd recurl
    cargo build --release

    # Binary location
    ./target/release/recurl --version
    ```

=== "macOS"

    ```bash
    # Install Xcode tools
    xcode-select --install

    # Clone and build
    git clone https://github.com/neul-labs/recurl.git
    cd recurl
    cargo build --release
    ```

=== "Windows"

    ```powershell
    # Requires Visual Studio Build Tools with C++ workload
    git clone https://github.com/neul-labs/recurl.git
    cd recurl
    cargo build --release

    # Binary location
    .\target\release\recurl.exe --version
    ```

### Run Tests

```bash
# All tests
cargo test

# Browser integration tests (requires Chromium)
cargo test --test browser_integration -- --test-threads=1
```

---

## Chromium Auto-Download

recurl automatically downloads Chromium on first JS preflight use. No manual installation required.

**Cache locations:**

| Platform | Path |
|----------|------|
| Linux | `~/.local/share/recurl/chromium/` |
| macOS | `~/Library/Application Support/recurl/chromium/` |
| Windows | `%LOCALAPPDATA%\recurl\chromium\` |

!!! note "Linux ARM64"
    Auto-download is not available for Linux ARM64. Install Chromium manually:

    ```bash
    # Ubuntu/Debian
    sudo apt install chromium-browser

    # Fedora
    sudo dnf install chromium

    # Arch
    sudo pacman -S chromium
    ```

---

## Verify Installation

```bash
# Check version
recurl --version

# Test with debug output
recurl --recurl-debug https://httpbin.org/get

# Test JS preflight (downloads Chromium if needed)
recurl --recurl-js --recurl-debug https://example.com
```

---

## Uninstall

### Linux / macOS

```bash
# Remove alias from shell config, then:
sudo rm -rf /usr/local/recurl

# Remove Chromium cache
rm -rf ~/.local/share/recurl  # Linux
rm -rf ~/Library/Application\ Support/recurl  # macOS
```

### Windows

1. Remove from PATH in Environment Variables
2. Delete `C:\Program Files\recurl\`
3. Remove alias from PowerShell profile
4. Delete `%LOCALAPPDATA%\recurl\`
