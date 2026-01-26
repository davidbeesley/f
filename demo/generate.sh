#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Get version from Cargo.toml
VERSION=$(grep '^version = ' "$PROJECT_ROOT/Cargo.toml" | head -1 | sed 's/version = "\(.*\)"/\1/')

if [[ -z "$VERSION" ]]; then
    echo "Error: Could not extract version from Cargo.toml"
    exit 1
fi

OUTPUT_FILE="$SCRIPT_DIR/demo-${VERSION}.gif"

echo "Generating demo for version $VERSION..."
echo "Output: $OUTPUT_FILE"

# Remove old demo GIFs
find "$SCRIPT_DIR" -name 'demo-*.gif' -delete 2>/dev/null || true

# Build the project first so demo uses current version
cargo build --release -p f --quiet

# Add to PATH for vhs
export PATH="$PROJECT_ROOT/target/release:$PATH"

# Generate the demo
cd "$SCRIPT_DIR"
vhs demo.tape -o "demo-${VERSION}.gif"

# Update README to reference new demo
sed -i "s|demo/demo-[0-9]*\.[0-9]*\.[0-9]*\.gif|demo/demo-${VERSION}.gif|g" "$PROJECT_ROOT/README.md"

echo "Demo generated: $OUTPUT_FILE"
echo "README.md updated to reference demo/demo-${VERSION}.gif"
