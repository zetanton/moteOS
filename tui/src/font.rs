// tui/src/font.rs
#![no_std]

use shared::FontError;

pub type Result<T> = core::result::Result<T, FontError>;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Pfs1Header {
    pub magic: [u8; 2],
    pub mode: u8,
    pub char_size: u8,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Pfs2Header {
    pub magic: [u8; 4],
    pub version: u32,
    pub header_size: u32,
    pub flags: u32,
    pub length: u32,
    pub char_size: u32,
    pub height: u32,
    pub width: u32,
}

pub enum Version {
    V1(Pfs1Header),
    V2(Pfs2Header),
}

pub struct Font {
    pub glyphs: &'static [u8],
    pub width: usize,
    pub height: usize,
    pub glyph_count: usize,
    pub header: Version,
}

impl Font {
    /// Loads a font from a byte slice.
    ///
    /// The input `data` must have a `'static` lifetime because the font glyphs are
    /// often embedded directly into the binary as static data (e.g., via `include_bytes!`).
    ///
    /// # Safety
    ///
    /// The byte slice must be a valid PSF font format.
    pub unsafe fn load_psf(data: &'static [u8]) -> Result<Self> {
        if data.len() < 2 {
            return Err(FontError::NotAPsfFont);
        }
        let magic1 = [data[0], data[1]];
        if magic1 == [0x36, 0x04] {
            let header = &*(data.as_ptr() as *const Pfs1Header);
            let glyphs = &data[core::mem::size_of::<Pfs1Header>()..];
            Ok(Font {
                glyphs,
                width: 8,
                height: header.char_size as usize,
                glyph_count: 256, // PSF1 usually has 256 or 512
                header: Version::V1(*header),
            })
        } else {
            if data.len() < 4 {
                return Err(FontError::NotAPsfFont);
            }
            let magic2 = [data[0], data[1], data[2], data[3]];
            if magic2 == [0x72, 0xb5, 0x4a, 0x86] {
                let header = &*(data.as_ptr() as *const Pfs2Header);
                let glyphs = &data[header.header_size as usize..];

                Ok(Font {
                    glyphs,
                    width: header.width as usize,
                    height: header.height as usize,
                    glyph_count: header.length as usize,
                    header: Version::V2(*header),
                })
            } else {
                Err(FontError::InvalidMagic)
            }
        }
    }

    pub fn glyph_data(&self, c: char) -> Option<&'static [u8]> {
        match self.header {
            Version::V1(header) => {
                let glyph_index = c as u32;
                if glyph_index >= 256 {
                    return None;
                }
                let glyph_start = glyph_index * header.char_size as u32;
                let glyph_end = glyph_start + header.char_size as u32;
                Some(&self.glyphs[glyph_start as usize..glyph_end as usize])
            }
            Version::V2(header) => {
                let glyph_index = if header.flags == 0 {
                    c as u32
                } else {
                    // TODO: Implement Unicode table lookup for PSF v2 fonts with a unicode table.
                    // For now, we just cast the character to a u32, which works for ASCII characters.
                    c as u32
                };

                if glyph_index >= header.length {
                    return None;
                }

                let glyph_start = glyph_index * header.char_size;
                let glyph_end = glyph_start + header.char_size;

                Some(&self.glyphs[glyph_start as usize..glyph_end as usize])
            }
        }
    }

    /// Renders a glyph into the provided buffer.
    /// Returns a slice of the buffer containing the glyph data if successful.
    pub fn render_glyph<'a>(&self, ch: char, buffer: &'a mut [u8]) -> Option<&'a [u8]> {
        let glyph = self.glyph_data(ch)?;
        if buffer.len() < glyph.len() {
            return None;
        }
        buffer[..glyph.len()].copy_from_slice(glyph);
        Some(&buffer[..glyph.len()])
    }
}
