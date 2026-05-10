#!/bin/bash
# Update package manifests with release checksums
# Usage: ./scripts/update-release.sh v0.1.2

set -e

VERSION="${1:-}"
if [ -z "$VERSION" ]; then
    echo "Usage: $0 <version>"
    echo "Example: $0 v0.1.2"
    exit 1
fi

# Strip 'v' prefix if present
VERSION_NUM="${VERSION#v}"

echo "Updating package manifests for version $VERSION_NUM..."

# GitHub release URL base
RELEASE_URL="https://github.com/neul-labs/recurl/releases/download/$VERSION"

# Download and compute checksums
echo "Downloading release artifacts..."

TEMP_DIR=$(mktemp -d)
trap "rm -rf $TEMP_DIR" EXIT

# Define platforms
declare -A PLATFORMS=(
    ["darwin-aarch64"]="recurl-darwin-aarch64.tar.gz"
    ["darwin-x86_64"]="recurl-darwin-x86_64.tar.gz"
    ["linux-aarch64"]="recurl-linux-aarch64.tar.gz"
    ["linux-x86_64"]="recurl-linux-x86_64.tar.gz"
    ["windows-x86_64"]="recurl-windows-x86_64.zip"
)

declare -A CHECKSUMS

for platform in "${!PLATFORMS[@]}"; do
    file="${PLATFORMS[$platform]}"
    echo "  Downloading $file..."

    if curl -sL "$RELEASE_URL/$file" -o "$TEMP_DIR/$file"; then
        checksum=$(sha256sum "$TEMP_DIR/$file" | cut -d' ' -f1)
        CHECKSUMS[$platform]="$checksum"
        echo "    SHA256: $checksum"
    else
        echo "    Warning: Failed to download $file"
    fi
done

# Update Homebrew formula
echo ""
echo "Updating Homebrew formula..."

HOMEBREW_FILE="packages/homebrew/recurl.rb"
if [ -f "$HOMEBREW_FILE" ]; then
    sed -i "s/version \"[^\"]*\"/version \"$VERSION_NUM\"/" "$HOMEBREW_FILE"

    if [ -n "${CHECKSUMS[darwin-aarch64]:-}" ]; then
        sed -i "s/PLACEHOLDER_SHA256_DARWIN_ARM64/${CHECKSUMS[darwin-aarch64]}/" "$HOMEBREW_FILE"
    fi
    if [ -n "${CHECKSUMS[darwin-x86_64]:-}" ]; then
        sed -i "s/PLACEHOLDER_SHA256_DARWIN_X64/${CHECKSUMS[darwin-x86_64]}/" "$HOMEBREW_FILE"
    fi
    if [ -n "${CHECKSUMS[linux-aarch64]:-}" ]; then
        sed -i "s/PLACEHOLDER_SHA256_LINUX_ARM64/${CHECKSUMS[linux-aarch64]}/" "$HOMEBREW_FILE"
    fi
    if [ -n "${CHECKSUMS[linux-x86_64]:-}" ]; then
        sed -i "s/PLACEHOLDER_SHA256_LINUX_X64/${CHECKSUMS[linux-x86_64]}/" "$HOMEBREW_FILE"
    fi

    echo "  Updated $HOMEBREW_FILE"
fi

# Update Scoop manifest
echo "Updating Scoop manifest..."

SCOOP_FILE="packages/scoop/recurl.json"
if [ -f "$SCOOP_FILE" ]; then
    sed -i "s/\"version\": \"[^\"]*\"/\"version\": \"$VERSION_NUM\"/" "$SCOOP_FILE"
    sed -i "s|/v[0-9.]*-*[a-z0-9]*/|/$VERSION/|g" "$SCOOP_FILE"

    if [ -n "${CHECKSUMS[windows-x86_64]:-}" ]; then
        sed -i "s/PLACEHOLDER_SHA256_WINDOWS_X64/${CHECKSUMS[windows-x86_64]}/" "$SCOOP_FILE"
        # Also update existing SHA256 hashes
        sed -i "s/\"hash\": \"[a-f0-9]*\"/\"hash\": \"${CHECKSUMS[windows-x86_64]}\"/" "$SCOOP_FILE"
    fi

    echo "  Updated $SCOOP_FILE"
fi

# Update external Homebrew tap
EXTERNAL_TAP_FILE="../homebrew-tap/Formula/recurl.rb"
if [ -f "$EXTERNAL_TAP_FILE" ]; then
    echo "Updating external Homebrew tap..."
    sed -i "s/version \"[^\"]*\"/version \"$VERSION_NUM\"/" "$EXTERNAL_TAP_FILE"
    sed -i "s|/v[0-9.]*-[a-z0-9]*/|/v$VERSION_NUM/|g" "$EXTERNAL_TAP_FILE"
    sed -i "s|/v[0-9.]*/|/v$VERSION_NUM/|g" "$EXTERNAL_TAP_FILE"

    if [ -n "${CHECKSUMS[darwin-aarch64]:-}" ]; then
        sed -i "s/PLACEHOLDER_SHA256_DARWIN_ARM64/${CHECKSUMS[darwin-aarch64]}/" "$EXTERNAL_TAP_FILE"
    fi
    if [ -n "${CHECKSUMS[darwin-x86_64]:-}" ]; then
        sed -i "s/PLACEHOLDER_SHA256_DARWIN_X64/${CHECKSUMS[darwin-x86_64]}/" "$EXTERNAL_TAP_FILE"
    fi
    if [ -n "${CHECKSUMS[linux-aarch64]:-}" ]; then
        sed -i "s/PLACEHOLDER_SHA256_LINUX_ARM64/${CHECKSUMS[linux-aarch64]}/" "$EXTERNAL_TAP_FILE"
    fi
    if [ -n "${CHECKSUMS[linux-x86_64]:-}" ]; then
        sed -i "s/PLACEHOLDER_SHA256_LINUX_X64/${CHECKSUMS[linux-x86_64]}/" "$EXTERNAL_TAP_FILE"
    fi

    echo "  Updated $EXTERNAL_TAP_FILE"
fi

echo ""
echo "Done! Review the changes and commit:"
echo "  git diff packages/"
echo "  git add packages/"
echo "  git commit -m 'Update package manifests for $VERSION'"
