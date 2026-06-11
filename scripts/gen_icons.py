import struct, zlib, os

def create_png(width, height, r, g, b):
    def chunk(chunk_type, data):
        c = chunk_type + data
        crc = struct.pack('>I', zlib.crc32(c) & 0xffffffff)
        return struct.pack('>I', len(data)) + c + crc
    sig = b'\x89PNG\r\n\x1a\n'
    header = struct.pack('>IIBBBBB', width, height, 8, 2, 0, 0, 0)
    ihdr = chunk(b'IHDR', header)
    raw = b''
    for y in range(height):
        raw += b'\x00'
        for x in range(width):
            raw += bytes([r, g, b])
    idat = chunk(b'IDAT', zlib.compress(raw))
    iend = chunk(b'IEND', b'')
    return sig + ihdr + idat + iend

def create_ico(png32):
    png_size = len(png32)
    entry = struct.pack('<BBBBHHII', 32, 32, 0, 0, 1, 32, png_size, 22)
    header = struct.pack('<HHH', 0, 1, 1)
    return header + entry + png32

base = 'src-tauri/icons'
os.makedirs(base, exist_ok=True)

icon32 = create_png(32, 32, 59, 130, 246)
icon128 = create_png(128, 128, 59, 130, 246)
icon256 = create_png(256, 256, 59, 130, 246)

with open(os.path.join(base, '32x32.png'), 'wb') as f: f.write(icon32)
with open(os.path.join(base, '128x128.png'), 'wb') as f: f.write(icon128)
with open(os.path.join(base, '128x128@2x.png'), 'wb') as f: f.write(icon256)
with open(os.path.join(base, 'icon.ico'), 'wb') as f: f.write(create_ico(icon32))
with open(os.path.join(base, 'icon.icns'), 'wb') as f: f.write(icon256)

print('Icons created successfully')
