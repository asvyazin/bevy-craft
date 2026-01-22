#!/bin/bash

# Script to generate SVG icons with item representations

SVG_DIR="assets/textures/svg"
PNG_DIR="assets/textures"

mkdir -p "$SVG_DIR"

# Create SVG file for each icon with item representation
create_svg_icon() {
    local name=$1
    local color=$2
    local svg_file="${SVG_DIR}/${name}.svg"
    
    case "$name" in
        "grass_icon")
            cat > "$svg_file" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 48 48">
    <rect x="2" y="2" width="44" height="44" rx="4" fill="${color}" stroke="#000" stroke-width="2"/>
    <rect x="8" y="26" width="32" height="16" fill="#795548"/>
    <path d="M12 26 L14 18 L18 26 L20 16 L24 26 L26 18 L30 26 L32 20 L36 26" fill="#4CAF50" stroke="#2E7D32" stroke-width="1"/>
</svg>
EOF
            ;;
        "dirt_icon")
            cat > "$svg_file" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 48 48">
    <rect x="2" y="2" width="44" height="44" rx="4" fill="${color}" stroke="#000" stroke-width="2"/>
    <circle cx="12" cy="16" r="3" fill="#5D4037" opacity="0.5"/>
    <circle cx="28" cy="24" r="2.5" fill="#5D4037" opacity="0.5"/>
    <circle cx="38" cy="14" r="2" fill="#5D4037" opacity="0.5"/>
    <circle cx="20" cy="32" r="3" fill="#5D4037" opacity="0.5"/>
    <circle cx="34" cy="34" r="2" fill="#5D4037" opacity="0.5"/>
    <circle cx="14" cy="38" r="2.5" fill="#5D4037" opacity="0.5"/>
</svg>
EOF
            ;;
        "stone_icon")
            cat > "$svg_file" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 48 48">
    <rect x="2" y="2" width="44" height="44" rx="4" fill="${color}" stroke="#000" stroke-width="2"/>
    <rect x="10" y="10" width="12" height="10" fill="#616161" opacity="0.5"/>
    <rect x="26" y="14" width="14" height="12" fill="#616161" opacity="0.5"/>
    <rect x="14" y="28" width="10" height="12" fill="#616161" opacity="0.5"/>
    <rect x="28" y="30" width="12" height="10" fill="#616161" opacity="0.5"/>
</svg>
EOF
            ;;
        "wood_icon")
            cat > "$svg_file" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 48 48">
    <rect x="2" y="2" width="44" height="44" rx="4" fill="${color}" stroke="#000" stroke-width="2"/>
    <line x1="8" y1="12" x2="40" y2="12" stroke="#6D4C41" stroke-width="2" opacity="0.5"/>
    <line x1="8" y1="18" x2="40" y2="18" stroke="#6D4C41" stroke-width="2" opacity="0.5"/>
    <line x1="8" y1="24" x2="40" y2="24" stroke="#6D4C41" stroke-width="2" opacity="0.5"/>
    <line x1="8" y1="30" x2="40" y2="30" stroke="#6D4C41" stroke-width="2" opacity="0.5"/>
    <line x1="8" y1="36" x2="40" y2="36" stroke="#6D4C41" stroke-width="2" opacity="0.5"/>
</svg>
EOF
            ;;
        "leaves_icon")
            cat > "$svg_file" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 48 48">
    <rect x="2" y="2" width="44" height="44" rx="4" fill="${color}" stroke="#000" stroke-width="2"/>
    <circle cx="14" cy="16" r="5" fill="#388E3C"/>
    <circle cx="34" cy="16" r="5" fill="#388E3C"/>
    <circle cx="24" cy="24" r="6" fill="#388E3C"/>
    <circle cx="14" cy="32" r="5" fill="#388E3C"/>
    <circle cx="34" cy="32" r="5" fill="#388E3C"/>
    <circle cx="24" cy="36" r="4" fill="#388E3C"/>
</svg>
EOF
            ;;
        "sand_icon")
            cat > "$svg_file" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 48 48">
    <rect x="2" y="2" width="44" height="44" rx="4" fill="${color}" stroke="#000" stroke-width="2"/>
    <circle cx="12" cy="14" r="2" fill="#F9A825" opacity="0.6"/>
    <circle cx="28" cy="12" r="1.5" fill="#F9A825" opacity="0.6"/>
    <circle cx="36" cy="18" r="2" fill="#F9A825" opacity="0.6"/>
    <circle cx="18" cy="22" r="1.5" fill="#F9A825" opacity="0.6"/>
    <circle cx="32" cy="26" r="2" fill="#F9A825" opacity="0.6"/>
    <circle cx="14" cy="32" r="2" fill="#F9A825" opacity="0.6"/>
    <circle cx="26" cy="34" r="1.5" fill="#F9A825" opacity="0.6"/>
    <circle cx="38" cy="36" r="2" fill="#F9A825" opacity="0.6"/>
    <circle cx="20" cy="40" r="1.5" fill="#F9A825" opacity="0.6"/>
</svg>
EOF
            ;;
        "bedrock_icon")
            cat > "$svg_file" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 48 48">
    <rect x="2" y="2" width="44" height="44" rx="4" fill="${color}" stroke="#000" stroke-width="2"/>
    <rect x="6" y="6" width="16" height="14" fill="#424242" opacity="0.4"/>
    <rect x="26" y="6" width="16" height="12" fill="#424242" opacity="0.4"/>
    <rect x="8" y="26" width="14" height="14" fill="#424242" opacity="0.4"/>
    <rect x="26" y="24" width="16" height="18" fill="#424242" opacity="0.4"/>
    <line x1="6" y1="6" x2="22" y2="20" stroke="#616161" stroke-width="1" opacity="0.5"/>
    <line x1="26" y1="6" x2="42" y2="18" stroke="#616161" stroke-width="1" opacity="0.5"/>
</svg>
EOF
            ;;
        "water_icon")
            cat > "$svg_file" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 48 48">
    <rect x="2" y="2" width="44" height="44" rx="4" fill="${color}" stroke="#000" stroke-width="2"/>
    <path d="M6 18 Q14 14 22 18 T38 18" fill="none" stroke="#1565C0" stroke-width="2" opacity="0.6"/>
    <path d="M6 24 Q14 20 22 24 T38 24" fill="none" stroke="#1565C0" stroke-width="2" opacity="0.6"/>
    <path d="M6 30 Q14 26 22 30 T38 30" fill="none" stroke="#1565C0" stroke-width="2" opacity="0.6"/>
    <path d="M6 36 Q14 32 22 36 T38 36" fill="none" stroke="#1565C0" stroke-width="2" opacity="0.6"/>
</svg>
EOF
            ;;
        "glass_icon")
            cat > "$svg_file" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 48 48">
    <rect x="2" y="2" width="44" height="44" rx="4" fill="${color}" stroke="#000" stroke-width="2"/>
    <line x1="6" y1="12" x2="42" y2="12" stroke="#64B5F6" stroke-width="1" opacity="0.7"/>
    <line x1="6" y1="18" x2="42" y2="18" stroke="#64B5F6" stroke-width="1" opacity="0.7"/>
    <line x1="6" y1="24" x2="42" y2="24" stroke="#64B5F6" stroke-width="1" opacity="0.7"/>
    <line x1="6" y1="30" x2="42" y2="30" stroke="#64B5F6" stroke-width="1" opacity="0.7"/>
    <line x1="6" y1="36" x2="42" y2="36" stroke="#64B5F6" stroke-width="1" opacity="0.7"/>
    <line x1="12" y1="6" x2="12" y2="42" stroke="#64B5F6" stroke-width="1" opacity="0.7"/>
    <line x1="24" y1="6" x2="24" y2="42" stroke="#64B5F6" stroke-width="1" opacity="0.7"/>
    <line x1="36" y1="6" x2="36" y2="42" stroke="#64B5F6" stroke-width="1" opacity="0.7"/>
</svg>
EOF
            ;;
        "brick_icon")
            cat > "$svg_file" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 48 48">
    <rect x="2" y="2" width="44" height="44" rx="4" fill="${color}" stroke="#000" stroke-width="2"/>
    <rect x="6" y="8" width="10" height="6" fill="#BF360C" stroke="#000" stroke-width="1"/>
    <rect x="18" y="8" width="10" height="6" fill="#BF360C" stroke="#000" stroke-width="1"/>
    <rect x="30" y="8" width="12" height="6" fill="#BF360C" stroke="#000" stroke-width="1"/>
    <rect x="6" y="16" width="12" height="6" fill="#BF360C" stroke="#000" stroke-width="1"/>
    <rect x="20" y="16" width="10" height="6" fill="#BF360C" stroke="#000" stroke-width="1"/>
    <rect x="32" y="16" width="10" height="6" fill="#BF360C" stroke="#000" stroke-width="1"/>
    <rect x="6" y="24" width="10" height="6" fill="#BF360C" stroke="#000" stroke-width="1"/>
    <rect x="18" y="24" width="10" height="6" fill="#BF360C" stroke="#000" stroke-width="1"/>
    <rect x="30" y="24" width="12" height="6" fill="#BF360C" stroke="#000" stroke-width="1"/>
    <rect x="6" y="32" width="12" height="6" fill="#BF360C" stroke="#000" stroke-width="1"/>
    <rect x="20" y="32" width="10" height="6" fill="#BF360C" stroke="#000" stroke-width="1"/>
    <rect x="32" y="32" width="10" height="6" fill="#BF360C" stroke="#000" stroke-width="1"/>
</svg>
EOF
            ;;
        "coal_icon")
            cat > "$svg_file" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 48 48">
    <rect x="2" y="2" width="44" height="44" rx="4" fill="${color}" stroke="#000" stroke-width="2"/>
    <ellipse cx="24" cy="24" rx="14" ry="12" fill="#37474F" stroke="#000" stroke-width="1"/>
    <ellipse cx="20" cy="20" rx="4" ry="3" fill="#102027"/>
    <ellipse cx="28" cy="22" rx="3" ry="2" fill="#102027"/>
    <ellipse cx="22" cy="28" rx="3" ry="2" fill="#102027"/>
    <ellipse cx="30" cy="27" rx="2" ry="2" fill="#102027"/>
</svg>
EOF
            ;;
        "gold_ingot_icon")
            cat > "$svg_file" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 48 48">
    <rect x="2" y="2" width="44" height="44" rx="4" fill="${color}" stroke="#000" stroke-width="2"/>
    <polygon points="14,16 34,16 38,28 10,28" fill="#FFC107" stroke="#000" stroke-width="1"/>
    <polygon points="34,16 38,28 34,32 30,28" fill="#FFA000"/>
    <rect x="14" y="32" width="16" height="2" fill="#FFA000"/>
</svg>
EOF
            ;;
        "iron_ingot_icon")
            cat > "$svg_file" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 48 48">
    <rect x="2" y="2" width="44" height="44" rx="4" fill="${color}" stroke="#000" stroke-width="2"/>
    <polygon points="14,16 34,16 38,28 10,28" fill="#B0BEC5" stroke="#000" stroke-width="1"/>
    <polygon points="34,16 38,28 34,32 30,28" fill="#78909C"/>
    <rect x="14" y="32" width="16" height="2" fill="#78909C"/>
</svg>
EOF
            ;;
        "stick_icon")
            cat > "$svg_file" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 48 48">
    <rect x="2" y="2" width="44" height="44" rx="4" fill="${color}" stroke="#000" stroke-width="2"/>
    <rect x="22" y="10" width="4" height="28" fill="#6D4C41" stroke="#4E342E" stroke-width="1"/>
    <rect x="22" y="10" width="4" height="4" fill="#8D6E63"/>
</svg>
EOF
            ;;
        "string_icon")
            cat > "$svg_file" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 48 48">
    <rect x="2" y="2" width="44" height="44" rx="4" fill="${color}" stroke="#000" stroke-width="2"/>
    <path d="M10 14 Q14 10 18 14 T26 14 T34 14" fill="none" stroke="#424242" stroke-width="2"/>
    <path d="M8 22 Q14 18 20 22 T32 22 T42 22" fill="none" stroke="#424242" stroke-width="2"/>
    <path d="M10 30 Q14 26 18 30 T26 30 T34 30" fill="none" stroke="#424242" stroke-width="2"/>
    <path d="M8 38 Q14 34 20 38 T32 38 T42 38" fill="none" stroke="#424242" stroke-width="2"/>
</svg>
EOF
            ;;
        "pickaxe_icon")
            cat > "$svg_file" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 48 48">
    <rect x="2" y="2" width="44" height="44" rx="4" fill="${color}" stroke="#000" stroke-width="2"/>
    <rect x="22" y="14" width="4" height="28" fill="#8D6E63" stroke="#5D4037" stroke-width="1"/>
    <rect x="10" y="10" width="6" height="8" fill="#78909C" stroke="#455A64" stroke-width="1"/>
    <rect x="32" y="10" width="6" height="8" fill="#78909C" stroke="#455A64" stroke-width="1"/>
    <polygon points="10,10 16,10 13,4 13,10" fill="#78909C" stroke="#455A64" stroke-width="1"/>
    <polygon points="32,10 38,10 35,4 35,10" fill="#78909C" stroke="#455A64" stroke-width="1"/>
</svg>
EOF
            ;;
        "axe_icon")
            cat > "$svg_file" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 48 48">
    <rect x="2" y="2" width="44" height="44" rx="4" fill="${color}" stroke="#000" stroke-width="2"/>
    <rect x="22" y="14" width="4" height="28" fill="#8D6E63" stroke="#5D4037" stroke-width="1"/>
    <polygon points="28,10 42,14 42,24 28,20" fill="#78909C" stroke="#455A64" stroke-width="1"/>
    <rect x="24" y="14" width="4" height="6" fill="#8D6E63"/>
</svg>
EOF
            ;;
        "shovel_icon")
            cat > "$svg_file" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 48 48">
    <rect x="2" y="2" width="44" height="44" rx="4" fill="${color}" stroke="#000" stroke-width="2"/>
    <rect x="22" y="10" width="4" height="32" fill="#8D6E63" stroke="#5D4037" stroke-width="1"/>
    <rect x="16" y="38" width="16" height="4" fill="#78909C" stroke="#455A64" stroke-width="1"/>
</svg>
EOF
            ;;
        "sword_icon")
            cat > "$svg_file" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 48 48">
    <rect x="2" y="2" width="44" height="44" rx="4" fill="${color}" stroke="#000" stroke-width="2"/>
    <rect x="22" y="8" width="4" height="36" fill="#78909C" stroke="#455A64" stroke-width="1"/>
    <rect x="16" y="36" width="16" height="4" fill="#8D6E63" stroke="#5D4037" stroke-width="1"/>
    <polygon points="22,8 26,8 24,2" fill="#90A4AE"/>
</svg>
EOF
            ;;
        "unknown_icon")
            cat > "$svg_file" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 48 48">
    <rect x="2" y="2" width="44" height="44" rx="4" fill="${color}" stroke="#000" stroke-width="2"/>
    <text x="24" y="36" font-family="Arial, sans-serif" font-size="28" text-anchor="middle" fill="#000" font-weight="bold">?</text>
</svg>
EOF
            ;;
        *)
            cat > "$svg_file" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 48 48">
    <rect x="2" y="2" width="44" height="44" rx="4" fill="${color}" stroke="#000" stroke-width="2"/>
</svg>
EOF
            ;;
    esac
    echo "Created $svg_file"
}

# Create all SVG icons with item representations
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

echo "All SVG icons with item representations created successfully!"

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
