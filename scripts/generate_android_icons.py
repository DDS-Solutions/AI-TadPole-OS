"""
@docs ARCHITECTURE:Infrastructure:Scripts

### AI Context Alignment
- **Subsystem**: Developer Scripts / generate_android_icons
- **Primary Entrypoints**: `generate_android_icons`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: none declared
- **Witness Tests**: none declared
"""

# [android_icons] Android icon generator script
import os
from pathlib import Path
from PIL import Image, ImageDraw


ROOT = Path(__file__).resolve().parent.parent

def generate_android_icons():
    source_path = ROOT / "public" / "assets" / "logo.png"
    base_res_dir = ROOT / "apps" / "mobile-android" / "app" / "src" / "main" / "res"

    if not os.path.exists(source_path):
        print(f"Error: Source logo not found at {source_path}")
        return

    src_img = Image.open(source_path).convert("RGBA")

    sizes = {
        "mipmap-mdpi": 48,
        "mipmap-hdpi": 72,
        "mipmap-xhdpi": 96,
        "mipmap-xxhdpi": 144,
        "mipmap-xxxhdpi": 192
    }

    def make_full_fill_icon(canvas_size):
        """Resizes logo to fill 100% edge-to-edge of icon canvas."""
        return src_img.resize((canvas_size, canvas_size), Image.Resampling.LANCZOS)

    def make_circular_icon(icon_img):
        """Clips icon into smooth circular mask for Android roundIcon."""
        size = icon_img.size[0]
        mask = Image.new('L', (size, size), 0)
        draw = ImageDraw.Draw(mask)
        draw.ellipse((0, 0, size - 1, size - 1), fill=255)
        
        output = Image.new('RGBA', (size, size), (0, 0, 0, 0))
        output.paste(icon_img, (0, 0), mask=mask)
        return output

    for folder, size in sizes.items():
        folder_path = os.path.join(base_res_dir, folder)
        os.makedirs(folder_path, exist_ok=True)

        # 100% Full-bleed fill icon
        full_icon = make_full_fill_icon(size)
        icon_path = os.path.join(folder_path, "ic_launcher.png")
        full_icon.save(icon_path, "PNG")

        # Full-bleed circular icon
        round_icon = make_circular_icon(full_icon)
        round_icon_path = os.path.join(folder_path, "ic_launcher_round.png")
        round_icon.save(round_icon_path, "PNG")

        print(f"Generated 100% full-bleed fill {size}x{size} icon in {folder}")

    # Generate extra high-resolution store & master icons (100% full fill)
    extra_sizes = {
        "ic_launcher_256.png": 256,
        "ic_launcher_512.png": 512,
        "ic_launcher_1024.png": 1024
    }

    drawable_dir = os.path.join(base_res_dir, "drawable")
    os.makedirs(drawable_dir, exist_ok=True)

    for filename, size in extra_sizes.items():
        full_icon = make_full_fill_icon(size)
        output_path = os.path.join(drawable_dir, filename)
        full_icon.save(output_path, "PNG")
        print(f"Generated full-bleed {size}x{size} icon at drawable/{filename}")

if __name__ == "__main__":
    generate_android_icons()
