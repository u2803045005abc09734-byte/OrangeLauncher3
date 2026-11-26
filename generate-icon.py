from pathlib import Path
from PIL import Image

icons_dir = Path("packages/oneclient_app/icons")
source_path = icons_dir / "icon.png"
image = Image.open(source_path).convert("RGBA")
if image.size != (512, 512):
    image = image.resize((512, 512), Image.Resampling.LANCZOS)

# Rebuild every Windows-visible launcher icon from the Orange Client source image.
# In particular, 128x128.png is used for the launcher and social-window title bars.
png_sizes = {
    "32x32.png": 32,
    "64x64.png": 64,
    "128x128.png": 128,
    "128x128@2x.png": 256,
    "512x512.png": 512,
    "StoreLogo.png": 150,
    "tray.png": 32,
}
for filename, size in png_sizes.items():
    resized = image.resize((size, size), Image.Resampling.LANCZOS)
    resized.save(icons_dir / filename, format="PNG")

image.save(
    icons_dir / "icon.ico",
    format="ICO",
    sizes=[(256, 256), (128, 128), (64, 64), (48, 48), (32, 32), (16, 16)],
)
print("Generated Orange Client PNG, ICO, taskbar and tray icons")
