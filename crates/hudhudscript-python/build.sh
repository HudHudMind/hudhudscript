#!/bin/bash
# Build HudHudScript Python module

set -e

echo "🔨 Building HudHudScript Python Module"
echo ""

# Check if we're in a virtual environment
if [ -z "$VIRTUAL_ENV" ]; then
    echo "⚠️  Not in a virtual environment"
    echo ""
    echo "Creating virtual environment..."
    python3 -m venv venv
    echo "Activating virtual environment..."
    source venv/bin/activate
    echo "✓ Virtual environment activated"
    echo ""
fi

# Check if maturin is installed
if ! command -v maturin &> /dev/null; then
    echo "📦 Installing maturin..."
    pip install maturin
fi

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust is not installed!"
    echo "   Install from: https://rustup.rs/"
    exit 1
fi

echo "🔧 Building with maturin..."
maturin develop --release

echo ""
echo "✅ Build complete!"
echo ""
echo "Test the module:"
if [ -n "$VIRTUAL_ENV" ]; then
    echo "  python -c 'import hudhudscript; print(hudhudscript.version())'"
else
    echo "  python3 -c 'import hudhudscript; print(hudhudscript.version())'"
fi
echo ""
