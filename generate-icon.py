from pathlib import Path
from PIL import Image

icon_path = Path("packages/oneclient_app/icons/icon.png")
ico_path = Path("packages/oneclient_app/icons/icon.ico")

image = Image.open(icon_path).convert("RGBA")
if image.size != (512, 512):
    image = image.resize((512, 512), Image.Resampling.LANCZOS)

image.save(
    ico_path,
    format="ICO",
    sizes=[(256, 256), (128, 128), (64, 64), (48, 48), (32, 32), (16, 16)],
)
print(f"Generated {ico_path}")
