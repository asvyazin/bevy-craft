#!/bin/bash

# Script to generate SVG icons (without text) and convert to PNG using ImageMagick

SVG_DIR="assets/textures/svg"
PNG_DIR="assets/textures"

mkdir -p "$SVG_DIR"

# Create SVG file for each icon (using only shapes, no text)
create_svg_icon() {
    local name=$1
    local color=$2
    local svg_file="${SVG_DIR}/${name}.svg"
    local png_file="${PNG_DIR}/${name}.png"
    
    # Create a simple SVG with a colored rectangle and border
    cat > "$svg_file" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 48 48">
    <rect x="2" y="2" width="44" height="44" rx="4" fill="${color}" stroke="#000" stroke-width="2"/>
</svg>
EOF
    echo "Created $svg_file"
}

# Create all SVG icons
create_svg_icon "grass_icon" "#8BC34A"
create_svg_icon "dirt_icon" "#795548"
create_svg_icon "stone_icon" "#9E9E9E"
create_svg_icon "wood_icon" "#8D6E63"
create_svg_icon "leaves_icon" "#4CAF50"
create_svg_icon "sand_icon" "#FFEB3B"
create_svg_icon "bedrock_icon" "#212121"
create_svg_icon "water_icon" "#2196F3"
create_svg_icon "glass_icon" "#90CAF9"
create_svg_icon "brick_icon" "#D84315"
create_svg_icon "coal_icon" "#263238"
create_svg_icon "gold_ingot_icon" "#FFD54F"
create_svg_icon "iron_ingot_icon" "#B0BEC5"
create_svg_icon "stick_icon" "#A1887F"
create_svg_icon "string_icon" "#F5F5F5"
create_svg_icon "pickaxe_icon" "#607D8B"
create_svg_icon "axe_icon" "#5D4037"
create_svg_icon "shovel_icon" "#795548"
create_svg_icon "sword_icon" "#424242"
create_svg_icon "unknown_icon" "#616161"

echo "All SVG icons created successfully!"

# Now convert SVG to PNG
echo ""
echo "Converting SVG icons to PNG..."

for svg_file in "$SVG_DIR"/*.svg; do
    if [ -f "$svg_file" ]; then
        filename=$(basename "$svg_file" .svg)
        png_file="${PNG_DIR}/${filename}.png"
        
        echo "Converting $svg_file to $png_file..."
        magick "$svg_file" "$png_file"
        
        if [ $? -eq 0 ]; then
            echo "Successfully converted $filename"
        else
            echo "Failed to convert $filename"
        fi
    fi
done

echo "All icons converted to PNG!"
