# Quick Start

Get recurl running in under 2 minutes.

---

## Install

=== "Linux / macOS"

    ```bash
    curl -fsSL https://recurl.dev/install.sh | bash
    ```

=== "Windows (PowerShell)"

    ```powershell
    irm https://recurl.dev/install.ps1 | iex
    ```

=== "Homebrew"

    ```bash
    brew install recurl/tap/recurl
    ```

=== "Docker"

    ```bash
    docker run --rm ghcr.io/user/recurl https://example.com
    ```

---

## Verify Installation

```bash
recurl --version
```

You should see the curl version info (recurl uses a real curl engine).

---

## Make Your First Request

```bash
# Basic request
recurl https://httpbin.org/get

# With debug output to see what's happening
recurl --recurl-debug https://httpbin.org/get
```

---

## Set Up the curl Alias (Optional)

The installer will ask if you want to alias `curl` to `recurl`. If you said no but want to enable it later:

=== "Bash"

    ```bash
    echo 'alias curl="/usr/local/recurl/recurl"' >> ~/.bashrc
    source ~/.bashrc
    ```

=== "Zsh"

    ```bash
    echo 'alias curl="/usr/local/recurl/recurl"' >> ~/.zshrc
    source ~/.zshrc
    ```

=== "PowerShell"

    ```powershell
    Add-Content $PROFILE 'Set-Alias -Name curl -Value "C:\Program Files\recurl\recurl.exe"'
    . $PROFILE
    ```

Now `curl` commands automatically use recurl:

```bash
curl https://protected-site.com  # Uses recurl transparently
```

---

## Test Anti-Bot Bypass

Try a site that blocks regular curl:

```bash
# This might be blocked with regular curl
recurl --recurl-debug https://nowsecure.nl

# Output shows the escalation:
# [recurl] curl_engine: 403 Cloudflare
# [recurl] Escalating: impersonation (chrome)
# [recurl] curl_chrome: 200 OK
```

---

## What's Next?

- [Installation Details](installation.md) - Platform-specific options
- [CLI Reference](../usage/cli.md) - All available flags
- [How It Works](../how-it-works/architecture.md) - Under the hood
