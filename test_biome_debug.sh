#!/bin/bash

# Test script to verify biome debug system integration

echo "🔍 Testing biome debug system integration..."

cd /Volumes/KINGSTON/dev/rust/bevy-craft

# Check if biome_debug.rs exists and has the right content
echo "✓ Checking biome_debug.rs file..."
if [ -f "src/biome_debug.rs" ]; then
    echo "  ✓ biome_debug.rs file exists"
    
    # Check for key components
    if grep -q "BiomeDebugSettings" src/biome_debug.rs; then
        echo "  ✓ BiomeDebugSettings struct found"
    else
        echo "  ✗ BiomeDebugSettings struct not found"
        exit 1
    fi
    
    if grep -q "toggle_biome_debug" src/biome_debug.rs; then
        echo "  ✓ toggle_biome_debug function found"
    else
        echo "  ✗ toggle_biome_debug function not found"
        exit 1
    fi
    
    if grep -q "display_biome_debug_info" src/biome_debug.rs; then
        echo "  ✓ display_biome_debug_info function found"
    else
        echo "  ✗ display_biome_debug_info function not found"
        exit 1
    fi
else
    echo "  ✗ biome_debug.rs file not found"
    exit 1
fi

# Check if biome debug is integrated into main.rs
echo "✓ Checking main.rs integration..."
if grep -q "mod biome_debug" src/main.rs; then
    echo "  ✓ biome_debug module imported"
else
    echo "  ✗ biome_debug module not imported"
    exit 1
fi

if grep -q "BiomeDebugSettings" src/main.rs; then
    echo "  ✓ BiomeDebugSettings used in main.rs"
else
    echo "  ✗ BiomeDebugSettings not used in main.rs"
    exit 1
fi

if grep -q "initialize_biome_debug_system" src/main.rs; then
    echo "  ✓ initialize_biome_debug_system added to startup"
else
    echo "  ✗ initialize_biome_debug_system not added to startup"
    exit 1
fi

if grep -q "toggle_biome_debug" src/main.rs; then
    echo "  ✓ toggle_biome_debug added to update systems"
else
    echo "  ✗ toggle_biome_debug not added to update systems"
    exit 1
fi

# Test compilation
echo "✓ Testing compilation..."
if cargo check --quiet; then
    echo "  ✓ Project compiles successfully"
else
    echo "  ✗ Project compilation failed"
    exit 1
fi

echo ""
echo "🎉 All biome debug system integration tests passed!"
echo ""
echo "📋 Summary of implemented features:"
echo "  • BiomeDebugSettings resource for configuration"
echo "  • BiomeDebugStats resource for tracking statistics"
echo "  • Keyboard controls (F3, F4) for toggling debug modes"
echo "  • Console output for biome debug information"
echo "  • Framework for biome boundary visualization"
echo "  • Framework for biome texture variation visualization"
echo "  • Helper functions for biome color mapping"
echo ""
echo "🔧 Next steps for full implementation:"
echo "  • Implement actual biome boundary visualization"
echo "  • Implement biome texture variation visualization"
echo "  • Add UI elements for biome debug information"
echo "  • Integrate with biome texture cache statistics"
echo "  • Add more detailed biome parameter visualization"

echo ""
echo "💡 Usage:"
echo "  Press F3 to toggle biome debug visualization"
echo "  Press F4 to toggle advanced biome debugging"
echo "  Debug info will be displayed in console every 3 seconds"
