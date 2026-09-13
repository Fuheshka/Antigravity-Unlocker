#!/usr/bin/env python3
"""
Antigravity Unlocker - macOS Icon Generator
Converts an input 1024x1024 PNG into an Apple-compliant AppIcon.icns bundle.
"""

import os
import sys
import shutil
import subprocess
from PIL import Image

def main():
    root = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
    default_src = os.path.join(root, "assets", "AppIcon_v1_clean.png")
    src = sys.argv[1] if len(sys.argv) > 1 else default_src
    out_icns = os.path.join(root, "macos", "AppIcon.icns")

    if not os.path.exists(src):
        print(f"Error: source image not found: {src}", file=sys.stderr)
        sys.exit(1)

    iconset_dir = os.path.join(root, "macos", "AppIcon.iconset")
    os.makedirs(iconset_dir, exist_ok=True)

    img = Image.open(src).convert("RGBA")

    sizes = [
        (16, "icon_16x16.png"),
        (32, "icon_16x16@2x.png"),
        (32, "icon_32x32.png"),
        (64, "icon_32x32@2x.png"),
        (128, "icon_128x128.png"),
        (256, "icon_128x128@2x.png"),
        (256, "icon_256x256.png"),
        (512, "icon_256x256@2x.png"),
        (512, "icon_512x512.png"),
        (1024, "icon_512x512@2x.png"),
    ]

    print(f"==> Generating iconset from: {src}")
    for dim, fname in sizes:
        resized = img.resize((dim, dim), Image.Resampling.LANCZOS)
        resized.save(os.path.join(iconset_dir, fname))

    print("==> Compiling into ICNS with iconutil...")
    subprocess.run(["iconutil", "-c", "icns", iconset_dir, "-o", out_icns], check=True)
    shutil.rmtree(iconset_dir)
    print(f"==> Successfully built: {out_icns}")

if __name__ == "__main__":
    main()
