#!/usr/bin/env node
/**
 * Invert a PNG's color channels in place, preserving alpha.
 * Used to derive the light-appearance favicon from the monochrome
 * `Icon-iOS-Dark` export (the icon project only exports the dark look).
 *
 * Usage: node scripts/invert-png.mjs <source.png> <dest.png>
 * Supports 8-bit, non-interlaced PNGs (color types 0/2/3/4/6).
 */
import { readFileSync, writeFileSync } from 'node:fs';
import { deflateSync, inflateSync } from 'node:zlib';

const [src, dest] = process.argv.slice(2);
if (!src || !dest) {
    console.error('usage: node scripts/invert-png.mjs <source.png> <dest.png>');
    process.exit(1);
}

const buf = readFileSync(src);
const signature = Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]);
if (!buf.subarray(0, 8).equals(signature)) throw new Error('not a PNG');

const chunks = [];
for (let off = 8; off < buf.length;) {
    const len = buf.readUInt32BE(off);
    const type = buf.toString('latin1', off + 4, off + 8);
    chunks.push({ type, data: buf.subarray(off + 8, off + 8 + len) });
    off += 12 + len;
}

const ihdr = chunks.find((c) => c.type === 'IHDR').data;
const width = ihdr.readUInt32BE(0);
const height = ihdr.readUInt32BE(4);
const bitDepth = ihdr[8];
const colorType = ihdr[9];
if (ihdr[12] !== 0) throw new Error('interlaced PNGs are not supported');

let idat;
if (colorType === 3) {
    // Palette image: invert the palette entries, pass pixel data through.
    const plte = chunks.find((c) => c.type === 'PLTE').data;
    for (let i = 0; i < plte.length; i++) plte[i] = 255 - plte[i];
    idat = Buffer.concat(
        chunks.filter((c) => c.type === 'IDAT').map((c) => c.data),
    );
} else {
    if (![8, 16].includes(bitDepth)) {
        throw new Error(`bit depth ${bitDepth} is not supported`);
    }
    const channels = { 0: 1, 2: 3, 4: 2, 6: 4 }[colorType];
    if (!channels) throw new Error(`color type ${colorType} is not supported`);

    const bytesPerSample = bitDepth / 8;
    const stride = width * channels * bytesPerSample;
    const raw = inflateSync(
        Buffer.concat(
            chunks.filter((c) => c.type === 'IDAT').map((c) => c.data),
        ),
    );
    if (raw.length !== height * (stride + 1))
        throw new Error('unexpected IDAT size');

    // Undo the per-scanline PNG filters (None/Sub/Up/Average/Paeth).
    const img = Buffer.alloc(height * stride);
    const paeth = (a, b, c) => {
        const p = a + b - c;
        const pa = Math.abs(p - a);
        const pb = Math.abs(p - b);
        const pc = Math.abs(p - c);
        return pa <= pb && pa <= pc ? a : pb <= pc ? b : c;
    };
    for (let y = 0; y < height; y++) {
        const filter = raw[y * (stride + 1)];
        const row = raw.subarray(y * (stride + 1) + 1, (y + 1) * (stride + 1));
        const prev = y > 0 ? img.subarray((y - 1) * stride, y * stride) : null;
        const out = img.subarray(y * stride, (y + 1) * stride);
        // Neighboring pixels are `bpp` bytes apart (not one sample's bytes).
        const bpp = channels * bytesPerSample;
        for (let i = 0; i < stride; i++) {
            const a = i >= bpp ? out[i - bpp] : 0;
            const b = prev ? prev[i] : 0;
            const c = prev && i >= bpp ? prev[i - bpp] : 0;
            let v = row[i];
            if (filter === 1) v += a;
            else if (filter === 2) v += b;
            else if (filter === 3) v += (a + b) >> 1;
            else if (filter === 4) v += paeth(a, b, c);
            out[i] = v & 0xff;
        }
    }

    // Invert color channels, keep alpha. Samples are 1 or 2 bytes, big-endian.
    const alphaIndex = { 0: Infinity, 2: Infinity, 4: 1, 6: 3 }[colorType];
    for (let i = 0; i < img.length; i += bytesPerSample) {
        if (Math.floor(i / bytesPerSample) % channels === alphaIndex) continue;
        if (bytesPerSample === 1) {
            img[i] = 255 - img[i];
        } else {
            const value = (img[i] << 8) | img[i + 1];
            const inverted = 65535 - value;
            img[i] = inverted >> 8;
            img[i + 1] = inverted & 0xff;
        }
    }

    // Re-emit scanlines with filter 0 (None).
    const filtered = Buffer.alloc(height * (stride + 1));
    for (let y = 0; y < height; y++) {
        filtered[y * (stride + 1)] = 0;
        img.copy(filtered, y * (stride + 1) + 1, y * stride, (y + 1) * stride);
    }
    idat = deflateSync(filtered, { level: 9 });
}

const crcTable = new Int32Array(256);
for (let n = 0; n < 256; n++) {
    let c = n;
    for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
    crcTable[n] = c;
}
const crc32 = (bytes) => {
    let c = 0xffffffff;
    for (const byte of bytes) c = crcTable[(c ^ byte) & 0xff] ^ (c >>> 8);
    return (c ^ 0xffffffff) >>> 0;
};

const chunk = (type, data) => {
    const head = Buffer.alloc(4);
    head.writeUInt32BE(data.length, 0);
    const body = Buffer.concat([Buffer.from(type, 'latin1'), data]);
    const crc = Buffer.alloc(4);
    crc.writeUInt32BE(crc32(body), 0);
    return Buffer.concat([head, body, crc]);
};

const png = Buffer.concat([
    signature,
    chunk('IHDR', Buffer.from(ihdr)),
    ...chunks
        .filter((c) => c.type === 'PLTE' || c.type === 'tRNS')
        .map((c) => chunk(c.type, c.data)),
    chunk('IDAT', idat),
    chunk('IEND', Buffer.alloc(0)),
]);
writeFileSync(dest, png);
console.log(`wrote ${dest} (${png.length} bytes)`);
