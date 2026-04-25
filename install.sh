#!/bin/sh
set -e

# Configuration
REPO="hormuz-labs/puppetmaster"
BINARY_NAME="puppetmaster"
INSTALL_DIR="/usr/local/bin"

# Detect OS
OS="$(uname -s)"
case "${OS}" in
    Linux*)     PLATFORM="linux";;
    Darwin*)    PLATFORM="macos";;
    *)          echo "Unsupported OS: ${OS}"; exit 1;;
esac

# Detect Architecture
ARCH="$(uname -m)"
case "${ARCH}" in
    x86_64)     ARCH_TYPE="x86_64";;
    arm64|aarch64) ARCH_TYPE="aarch64";;
    *)          echo "Unsupported Architecture: ${ARCH}"; exit 1;;
esac

# Map to asset names used in GitHub Actions
# puppetmaster-linux-x86_64.tar.gz
# puppetmaster-macos-x86_64.tar.gz
# puppetmaster-macos-aarch64.tar.gz
ASSET_NAME="puppetmaster-${PLATFORM}-${ARCH_TYPE}"
EXTENSION="tar.gz"

if [ "${PLATFORM}" = "linux" ] && [ "${ARCH_TYPE}" = "aarch64" ]; then
    echo "Linux ARM64 is not currently supported in the release matrix."
    exit 1
fi

# Get latest release tag
echo "🔍 Finding latest release..."
LATEST_TAG=$(curl -s "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')

if [ -z "${LATEST_TAG}" ]; then
    echo "❌ Could not find latest release tag."
    exit 1
fi

DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${LATEST_TAG}/${ASSET_NAME}.${EXTENSION}"

echo "🚀 Installing ${BINARY_NAME} ${LATEST_TAG} for ${PLATFORM}-${ARCH_TYPE}..."
echo "📥 Downloading from ${DOWNLOAD_URL}..."

# Download and extract
TMP_DIR=$(mktemp -d)
curl -L "${DOWNLOAD_URL}" | tar xz -C "${TMP_DIR}"

# Install
echo "📦 Installing to ${INSTALL_DIR} (may require sudo)..."
if [ -w "${INSTALL_DIR}" ]; then
    mv "${TMP_DIR}/${BINARY_NAME}" "${INSTALL_DIR}/"
    mv "${TMP_DIR}/tg-notify" "${INSTALL_DIR}/"
else
    sudo mv "${TMP_DIR}/${BINARY_NAME}" "${INSTALL_DIR}/"
    sudo mv "${TMP_DIR}/tg-notify" "${INSTALL_DIR}/"
fi

# Install Skill
SKILLS_DIR="${HOME}/.agents/skills"
OPENCODE_SKILLS_DIR="${HOME}/.config/opencode/skills"

echo "🔧 Installing Agent Skill..."
mkdir -p "${SKILLS_DIR}"
mkdir -p "${OPENCODE_SKILLS_DIR}"

cp -r "${TMP_DIR}/skills/telegram-notify" "${SKILLS_DIR}/"
cp -r "${TMP_DIR}/skills/telegram-notify" "${OPENCODE_SKILLS_DIR}/"


# Cleanup
rm -rf "${TMP_DIR}"

echo "✅ Installed successfully!"
echo "🚀 Run '${BINARY_NAME}' to start the bot."
echo "🤖 The 'telegram-notify' skill is now available for your AI agents."
