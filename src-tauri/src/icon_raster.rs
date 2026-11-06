//! Залитый оранжевый кубик.

pub fn paint(size: u32) -> Vec<u8> {
    let s = size as f32;
    let mut pixels = vec![0u8; (size * size * 4) as usize];
    let orange = [238, 116, 38];
    let pip = [42, 22, 12];
    let center = s * 0.5;
    let many = size >= 24;
    let pip_r = if many { s * 0.058 } else { s * 0.11 };
    let offset = s * 0.15;
    let pips: &[(f32, f32)] = if many {
        &[
            (center - offset, center - offset),
            (center + offset, center - offset),
            (center, center),
            (center - offset, center + offset),
            (center + offset, center + offset),
        ]
    } else {
        &[(center, center)]
    };

    for y in 0..size {
        for x in 0..size {
            let px = x as f32 + 0.5;
            let py = y as f32 + 0.5;
            let mut pixel = [0u8; 4];
            over(
                &mut pixel,
                orange,
                coverage(sd_round_rect(
                    px,
                    py,
                    center,
                    center,
                    s * 0.42,
                    s * 0.42,
                    s * 0.16,
                )),
            );
            for (cx, cy) in pips {
                over(
                    &mut pixel,
                    pip,
                    coverage(sd_circle(px, py, *cx, *cy, pip_r)),
                );
            }
            let index = ((y * size + x) * 4) as usize;
            pixels[index..index + 4].copy_from_slice(&pixel);
        }
    }
    pixels
}

#[allow(dead_code)] // build.rs uses this to embed the exe icon
pub fn ico(images: &[(u32, Vec<u8>)]) -> Vec<u8> {
    let blobs: Vec<Vec<u8>> = images.iter().map(|(size, rgba)| dib(*size, rgba)).collect();
    let count = blobs.len() as u16;
    let mut out = Vec::new();
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&count.to_le_bytes());

    let mut offset = 6 + u32::from(count) * 16;
    for (image, blob) in images.iter().zip(&blobs) {
        let dim = if image.0 >= 256 { 0 } else { image.0 as u8 };
        out.push(dim);
        out.push(dim);
        out.push(0);
        out.push(0);
        out.extend_from_slice(&1u16.to_le_bytes());
        out.extend_from_slice(&32u16.to_le_bytes());
        out.extend_from_slice(&(blob.len() as u32).to_le_bytes());
        out.extend_from_slice(&offset.to_le_bytes());
        offset += blob.len() as u32;
    }
    for blob in blobs {
        out.extend_from_slice(&blob);
    }
    out
}

#[allow(dead_code)]
fn dib(size: u32, rgba: &[u8]) -> Vec<u8> {
    let xor = xor_bitmap(size, rgba);
    let mask = and_mask(size, rgba);
    let mut dib = Vec::with_capacity(40 + xor.len() + mask.len());
    dib.extend_from_slice(&40u32.to_le_bytes());
    dib.extend_from_slice(&(size as i32).to_le_bytes());
    dib.extend_from_slice(&((size as i32) * 2).to_le_bytes());
    dib.extend_from_slice(&1u16.to_le_bytes());
    dib.extend_from_slice(&32u16.to_le_bytes());
    dib.extend_from_slice(&0u32.to_le_bytes());
    dib.extend_from_slice(&((xor.len() + mask.len()) as u32).to_le_bytes());
    dib.extend_from_slice(&0u32.to_le_bytes());
    dib.extend_from_slice(&0u32.to_le_bytes());
    dib.extend_from_slice(&0u32.to_le_bytes());
    dib.extend_from_slice(&0u32.to_le_bytes());
    dib.extend_from_slice(&xor);
    dib.extend_from_slice(&mask);
    dib
}

#[allow(dead_code)]
fn xor_bitmap(size: u32, rgba: &[u8]) -> Vec<u8> {
    let width = size as usize;
    let mut out = Vec::with_capacity(width * width * 4);
    for y in (0..width).rev() {
        for x in 0..width {
            let index = (y * width + x) * 4;
            out.extend_from_slice(&[
                rgba[index + 2],
                rgba[index + 1],
                rgba[index],
                rgba[index + 3],
            ]);
        }
    }
    out
}

#[allow(dead_code)]
fn and_mask(size: u32, rgba: &[u8]) -> Vec<u8> {
    let width = size as usize;
    let row_bytes = (width + 31) / 32 * 4;
    let mut mask = vec![0u8; row_bytes * width];
    for y in 0..width {
        let source_y = width - 1 - y;
        for x in 0..width {
            let alpha = rgba[(source_y * width + x) * 4 + 3];
            if alpha < 128 {
                mask[y * row_bytes + x / 8] |= 1 << (7 - (x % 8));
            }
        }
    }
    mask
}

fn sd_round_rect(px: f32, py: f32, cx: f32, cy: f32, half_w: f32, half_h: f32, radius: f32) -> f32 {
    let dx = (px - cx).abs() - (half_w - radius);
    let dy = (py - cy).abs() - (half_h - radius);
    let outside = dx.max(0.0).hypot(dy.max(0.0));
    outside + dx.min(0.0).max(dy.min(0.0)) - radius
}

fn sd_circle(px: f32, py: f32, cx: f32, cy: f32, radius: f32) -> f32 {
    let dx = px - cx;
    let dy = py - cy;
    dx.hypot(dy) - radius
}

fn coverage(distance: f32) -> f32 {
    (0.5 - distance).clamp(0.0, 1.0)
}

fn over(dst: &mut [u8; 4], rgb: [u8; 3], alpha: f32) {
    let src_a = alpha.clamp(0.0, 1.0);
    if src_a <= 0.0 {
        return;
    }
    let dst_a = f32::from(dst[3]) / 255.0;
    let out_a = src_a + dst_a * (1.0 - src_a);
    if out_a <= 0.0 {
        *dst = [0, 0, 0, 0];
        return;
    }
    for channel in 0..3 {
        let mixed = (f32::from(rgb[channel]) * src_a
            + f32::from(dst[channel]) * dst_a * (1.0 - src_a))
            / out_a;
        dst[channel] = mixed.round().clamp(0.0, 255.0) as u8;
    }
    dst[3] = (out_a * 255.0).round().clamp(0.0, 255.0) as u8;
}

#[cfg(test)]
mod tests {
    use super::paint;

    #[test]
    fn center_pixel_is_the_pip() {
        let pixels = paint(32);
        let index = (16 * 32 + 16) * 4;
        assert!(pixels[index] < 80, "pip should be dark");
        assert!(pixels[index + 3] > 200);
    }

    #[test]
    fn corner_is_transparent() {
        let pixels = paint(32);
        assert_eq!(pixels[3], 0);
    }
}
