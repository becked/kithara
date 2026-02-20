"""Generate macOS-style app icon from kithara source image.

Creates a 1024x1024 icon with:
- Dark squircle background
- Upper portion of kithara zoomed in to fill the frame
- Pop-out effect: tuning pegs and arms extend beyond squircle boundary

Uses a two-layer approach:
- Inside the squircle: original source image (no masking artifacts)
- Outside the squircle: masked instrument only (clean pop-out)
"""

from PIL import Image, ImageDraw, ImageFilter
import numpy as np
from scipy import ndimage
import os

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
PROJECT_DIR = os.path.dirname(SCRIPT_DIR)

# === Config ===
ICON_SIZE = 1024
SHAPE_MARGIN = 82  # Padding around squircle (Apple standard ~8%)
SHAPE_SIZE = ICON_SIZE - 2 * SHAPE_MARGIN  # ~860
CORNER_RADIUS = int(SHAPE_SIZE * 0.223)  # Apple's corner radius ratio


def create_squircle_mask(size: int, margin: int, radius: int) -> Image.Image:
    """Create a macOS-style rounded rectangle mask."""
    mask = Image.new("L", (size, size), 0)
    draw = ImageDraw.Draw(mask)
    draw.rounded_rectangle(
        [margin, margin, margin + (size - 2 * margin), margin + (size - 2 * margin)],
        radius=radius,
        fill=255,
    )
    return mask


def sample_bg_color(src_np: np.ndarray) -> tuple[int, int, int, int]:
    """Sample the background color from the squircle interior of the source."""
    h, w = src_np.shape[:2]
    # Sample from left and right edges in the middle vertical region
    samples = []
    for y_pct in [0.25, 0.35, 0.45]:
        y = int(y_pct * h)
        for x in range(50, 80):
            samples.append(src_np[y, x, :3])
        for x in range(w - 80, w - 50):
            samples.append(src_np[y, x, :3])
    avg = np.mean(samples, axis=0).astype(int)
    return (int(avg[0]), int(avg[1]), int(avg[2]), 255)


def create_instrument_mask(src_np: np.ndarray) -> Image.Image:
    """Create a mask isolating the instrument from its dark background.

    Uses brightness deviation from background + color saturation.
    """
    r, g, b = src_np[:, :, 0].astype(float), src_np[:, :, 1].astype(float), src_np[:, :, 2].astype(float)
    brightness = 0.299 * r + 0.587 * g + 0.114 * b
    h, w = brightness.shape

    # Model background brightness per row from edge pixels
    bg_brightness = np.zeros(h)
    for y in range(h):
        left = brightness[y, 40:90].mean()
        right = brightness[y, w - 90 : w - 40].mean()
        bg_brightness[y] = (left + right) / 2
    bg_model = np.tile(bg_brightness.reshape(-1, 1), (1, w))

    # Brightness deviation from background
    deviation = np.abs(brightness - bg_model)

    # Color saturation (instrument = warm wood tones, background = neutral gray)
    mx = np.maximum(np.maximum(r, g), b)
    mn = np.minimum(np.minimum(r, g), b)
    sat = np.where(mx > 0, (mx - mn) / mx, 0)

    # Combined mask with stricter thresholds
    mask = (deviation > 22) | (sat > 0.15)

    # Exclude non-instrument regions
    mask[int(h * 0.72) :, :] = False  # bottom (text label)
    mask[: int(h * 0.12), :] = False  # top (UI chrome)
    mask[:, :30] = False  # left edge
    mask[:, w - 30 :] = False  # right edge

    # Morphological cleanup
    mask_img = Image.fromarray((mask * 255).astype(np.uint8), mode="L")

    # Close: fill small gaps inside the instrument
    mask_img = mask_img.filter(ImageFilter.MaxFilter(9))
    mask_img = mask_img.filter(ImageFilter.MinFilter(7))

    # Open: remove small noise
    mask_img = mask_img.filter(ImageFilter.MinFilter(5))
    mask_img = mask_img.filter(ImageFilter.MaxFilter(5))

    # Keep only the largest connected region
    mask_np = np.array(mask_img)
    mask_np = (mask_np > 128).astype(np.uint8)
    labeled, num_features = ndimage.label(mask_np)
    if num_features > 1:
        sizes = ndimage.sum(mask_np, labeled, range(1, num_features + 1))
        largest = np.argmax(sizes) + 1
        mask_np = ((labeled == largest) * 255).astype(np.uint8)
        print(f"  Kept largest region (removed {num_features - 1} small artifacts)")
    else:
        mask_np = mask_np * 255

    # Smooth edges for anti-aliasing
    mask_img = Image.fromarray(mask_np, mode="L")
    mask_img = mask_img.filter(ImageFilter.GaussianBlur(radius=2.0))

    return mask_img


def crop_region(src: Image.Image, mask_img: Image.Image, keep_ratio: float = 0.78):
    """Find instrument bounds and crop to upper portion.

    Returns (cropped_source, cropped_mask, bounds_info).
    """
    # Find bounds from the mask
    mask_np = np.array(mask_img)
    rows = np.any(mask_np > 30, axis=1)
    cols = np.any(mask_np > 30, axis=0)
    rmin, rmax = np.where(rows)[0][[0, -1]]
    cmin, cmax = np.where(cols)[0][[0, -1]]

    inst_height = rmax - rmin
    crop_y2 = rmin + int(inst_height * keep_ratio)

    pad = 8
    x1 = max(0, cmin - pad)
    y1 = max(0, rmin - pad)
    x2 = min(src.width, cmax + pad)
    y2 = crop_y2

    print(f"  Instrument bounds: ({cmin},{rmin}) to ({cmax},{rmax}) = {cmax-cmin}x{rmax-rmin}")
    print(f"  Cropping to upper {keep_ratio:.0%}: ({x1},{y1})-({x2},{y2}) = {x2-x1}x{y2-y1}")

    cropped_src = src.crop((x1, y1, x2, y2))
    cropped_mask = mask_img.crop((x1, y1, x2, y2))

    return cropped_src, cropped_mask


def create_icon(
    source_cropped: Image.Image,
    mask_cropped: Image.Image,
    bg_color: tuple[int, int, int, int],
) -> Image.Image:
    """Compose the icon using a two-layer approach:

    - Inside squircle: original source image (natural dark background)
    - Outside squircle: masked instrument only (clean pop-out)
    """
    # Scale to fill most of the icon (larger = more pop-out on sides)
    inst_w, inst_h = source_cropped.size
    target_w = int(ICON_SIZE * 0.93)
    scale = target_w / inst_w
    target_h = int(inst_h * scale)

    source_scaled = source_cropped.resize((target_w, target_h), Image.LANCZOS)
    mask_scaled = mask_cropped.resize((target_w, target_h), Image.LANCZOS)

    print(f"  Scaled to {target_w}x{target_h} (scale: {scale:.2f}x)")

    # Position: center horizontally
    x_off = (ICON_SIZE - target_w) // 2

    # Position vertically: top of instrument pops above squircle
    # Place so tuning pegs extend above squircle top edge
    pop_out_top = 40  # Desired pixels above squircle
    y_off = SHAPE_MARGIN - pop_out_top

    # Check if bottom extends too far below squircle
    bottom = y_off + target_h
    squircle_bottom = SHAPE_MARGIN + SHAPE_SIZE
    if bottom > squircle_bottom + 20:
        # Shift up a bit but keep pop-out
        y_off = squircle_bottom - target_h + 20

    side_pop = SHAPE_MARGIN - x_off
    print(f"  Position: ({x_off}, {y_off})")
    print(f"  Pop-out: {pop_out_top}px top, {side_pop}px sides")

    # === Build the icon ===
    squircle_mask = create_squircle_mask(ICON_SIZE, SHAPE_MARGIN, CORNER_RADIUS)
    sq_np = np.array(squircle_mask)

    # Layer 1: Squircle filled with sampled background color
    icon = Image.new("RGBA", (ICON_SIZE, ICON_SIZE), (0, 0, 0, 0))
    bg = Image.new("RGBA", (ICON_SIZE, ICON_SIZE), bg_color)
    bg_masked = Image.new("RGBA", (ICON_SIZE, ICON_SIZE), (0, 0, 0, 0))
    bg_masked.paste(bg, mask=squircle_mask)
    icon = Image.alpha_composite(icon, bg_masked)

    # Layer 2: Source image clipped to squircle (natural look inside the icon)
    source_on_canvas = Image.new("RGBA", (ICON_SIZE, ICON_SIZE), (0, 0, 0, 0))
    # Convert source to RGBA if needed
    src_rgba = source_scaled.convert("RGBA")
    source_on_canvas.paste(src_rgba, (x_off, y_off))
    # Clip to squircle
    src_canvas_np = np.array(source_on_canvas)
    src_canvas_np[:, :, 3] = np.minimum(src_canvas_np[:, :, 3], sq_np)
    source_clipped = Image.fromarray(src_canvas_np)
    icon = Image.alpha_composite(icon, source_clipped)

    # Layer 3: Masked instrument OUTSIDE squircle only (pop-out effect)
    # Create the masked instrument layer
    inst_rgba = source_scaled.convert("RGBA")
    inst_np = np.array(inst_rgba)
    mask_np = np.array(mask_scaled)
    inst_np[:, :, 3] = mask_np  # Apply instrument mask as alpha

    inst_on_canvas = Image.new("RGBA", (ICON_SIZE, ICON_SIZE), (0, 0, 0, 0))
    inst_layer = Image.fromarray(inst_np)
    inst_on_canvas.paste(inst_layer, (x_off, y_off), inst_layer)

    # Keep only pixels OUTSIDE the squircle
    outside_np = np.array(inst_on_canvas)
    inverted_sq = 255 - sq_np
    outside_np[:, :, 3] = (
        outside_np[:, :, 3].astype(float) * inverted_sq.astype(float) / 255
    ).astype(np.uint8)
    inst_outside = Image.fromarray(outside_np)

    icon = Image.alpha_composite(icon, inst_outside)

    return icon


def generate_all_sizes(master_icon: Image.Image, output_dir: str) -> None:
    """Generate all icon sizes needed for Tauri, plus .icns and .ico."""
    os.makedirs(output_dir, exist_ok=True)
    import subprocess
    import tempfile

    # PNG sizes for Tauri
    png_sizes = {
        "icon.png": 512,
        "32x32.png": 32,
        "64x64.png": 64,
        "128x128.png": 128,
        "128x128@2x.png": 256,
    }

    for filename, size in png_sizes.items():
        resized = master_icon.resize((size, size), Image.LANCZOS)
        path = os.path.join(output_dir, filename)
        resized.save(path)
        print(f"  {filename} ({size}x{size})")

    master_path = os.path.join(output_dir, "icon-1024.png")
    master_icon.save(master_path)
    print(f"  icon-1024.png (1024x1024)")

    # macOS .icns via iconutil
    iconset_dir = os.path.join(output_dir, "icon.iconset")
    os.makedirs(iconset_dir, exist_ok=True)
    icns_sizes = {
        "icon_16x16.png": 16,
        "icon_16x16@2x.png": 32,
        "icon_32x32.png": 32,
        "icon_32x32@2x.png": 64,
        "icon_128x128.png": 128,
        "icon_128x128@2x.png": 256,
        "icon_256x256.png": 256,
        "icon_256x256@2x.png": 512,
        "icon_512x512.png": 512,
        "icon_512x512@2x.png": 1024,
    }
    for filename, size in icns_sizes.items():
        resized = master_icon.resize((size, size), Image.LANCZOS)
        resized.save(os.path.join(iconset_dir, filename))

    icns_path = os.path.join(output_dir, "icon.icns")
    result = subprocess.run(
        ["iconutil", "-c", "icns", iconset_dir, "-o", icns_path],
        capture_output=True, text=True,
    )
    if result.returncode == 0:
        print(f"  icon.icns (macOS)")
    else:
        print(f"  icon.icns FAILED: {result.stderr}")

    # Clean up iconset directory
    import shutil
    shutil.rmtree(iconset_dir, ignore_errors=True)

    # Windows .ico (multi-resolution)
    ico_path = os.path.join(output_dir, "icon.ico")
    ico_sizes = [16, 24, 32, 48, 64, 128, 256]
    ico_images = []
    for size in ico_sizes:
        resized = master_icon.resize((size, size), Image.LANCZOS)
        ico_images.append(resized)
    ico_images[0].save(ico_path, format="ICO", sizes=[(s, s) for s in ico_sizes],
                        append_images=ico_images[1:])
    print(f"  icon.ico (Windows)")

    # Favicon
    favicon = master_icon.resize((32, 32), Image.LANCZOS)
    favicon_path = os.path.join(PROJECT_DIR, "static", "favicon.png")
    favicon.save(favicon_path)
    print(f"  static/favicon.png (32x32)")


def main():
    src_path = os.path.join(PROJECT_DIR, "new-image-2.png")
    icons_dir = os.path.join(PROJECT_DIR, "src-tauri", "icons")

    print("Phase 1: Loading and analyzing source...")
    src = Image.open(src_path).convert("RGBA")
    src_np = np.array(src).astype(float)

    # Sample background color from source
    bg_color = sample_bg_color(np.array(src))
    print(f"  Sampled background color: RGBA{bg_color}")

    print("\nPhase 2: Creating instrument mask...")
    mask_img = create_instrument_mask(src_np)

    # Save debug mask
    mask_img.save(os.path.join(PROJECT_DIR, "_debug_mask.png"))

    print("\nPhase 3: Cropping to upper portion...")
    source_cropped, mask_cropped = crop_region(src, mask_img, keep_ratio=0.78)

    print("\nPhase 4: Composing icon...")
    icon = create_icon(source_cropped, mask_cropped, bg_color)

    preview_path = os.path.join(PROJECT_DIR, "icon-preview.png")
    icon.save(preview_path)
    print(f"  Saved preview to icon-preview.png")

    print("\nPhase 5: Generating all sizes...")
    generate_all_sizes(icon, icons_dir)

    print("\nDone! Check icon-preview.png")
    print("Note: .icns and .ico files need to be regenerated separately.")


if __name__ == "__main__":
    main()
