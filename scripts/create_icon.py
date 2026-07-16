import struct, zlib
from pathlib import Path

SIZE = 512
pixels = bytearray()
for y in range(SIZE):
    for x in range(SIZE):
        # Rounded dark-navy tile with a subtle radial cyan glow.
        r0 = min(x, SIZE - 1 - x)
        r1 = min(y, SIZE - 1 - y)
        corner = min(r0, r1)
        outside = corner < 58 and ((58-r0) ** 2 + (58-r1) ** 2 > 58 ** 2)
        dx, dy = x - 290, y - 210
        glow = max(0.0, 1.0 - (dx * dx + dy * dy) ** 0.5 / 360.0)
        bg = (9 + int(5 * glow), 16 + int(15 * glow), 32 + int(22 * glow), 0 if outside else 255)

        # Thick open C ring.
        cx, cy = 250, 255
        d = ((x - cx) ** 2 + (y - cy) ** 2) ** 0.5
        angle_gap = x > 270 and abs(y - cy) < 104
        ring = 132 < d < 188 and not angle_gap

        # Brush handle and bright tip across the opening.
        handle = abs((y - 315) + 0.62 * (x - 294)) < 18 and 285 < x < 423 and 205 < y < 350
        tip = ((x - 420) / 34) ** 2 + ((y - 232) / 28) ** 2 < 1

        # Two small sparkles.
        sparkle1 = (abs(x - 370) < 8 and abs(y - 135) < 27) or (abs(y - 135) < 8 and abs(x - 370) < 27)
        sparkle2 = (abs(x - 410) < 5 and abs(y - 174) < 17) or (abs(y - 174) < 5 and abs(x - 410) < 17)

        if ring or handle or tip or sparkle1 or sparkle2:
            t = min(1.0, max(0.0, (x + y - 180) / 620.0))
            color = (int(66 + 102 * t), int(216 + 32 * t), int(255 - 158 * t), 255)
        else:
            color = bg
        pixels.extend(color)

def chunk(tag, data):
    return struct.pack('>I', len(data)) + tag + data + struct.pack('>I', zlib.crc32(tag + data) & 0xffffffff)

raw = b''.join(b'\x00' + pixels[y * SIZE * 4:(y + 1) * SIZE * 4] for y in range(SIZE))
png = b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', struct.pack('>IIBBBBB', SIZE, SIZE, 8, 6, 0, 0, 0)) + chunk(b'IDAT', zlib.compress(raw, 9)) + chunk(b'IEND', b'')
out = Path(__file__).resolve().parents[1] / 'src-tauri' / 'icons' / 'icon.png'
out.parent.mkdir(parents=True, exist_ok=True)
out.write_bytes(png)
print(out)
