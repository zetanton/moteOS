// font/src/psf.rs

use core::mem;



pub type Result<T> = core::result::Result<T, Error>;



#[derive(Debug)]

pub enum Error {

    NotAPsfFont,

    InvalidMagic,

}



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

    pub header: Version,

    pub glyphs: &'static [u8],

}



impl Font {

    /// Loads a font from a byte slice.

    ///

    /// # Safety

    ///

    /// The byte slice must be valid and static.

    pub unsafe fn load(bytes: &'static [u8]) -> Result<Font> {

        if bytes.len() < 2 {

            return Err(Error::NotAPsfFont);

        }

        let magic1 = [bytes[0], bytes[1]];

        if magic1 == [0x36, 0x04] {

            let header = &*(bytes.as_ptr() as *const Pfs1Header);

            let glyphs = &bytes[mem::size_of::<Pfs1Header>()..];

            Ok(Font {

                header: Version::V1(*header),

                glyphs,

            })

        } else {

            if bytes.len() < 4 {

                return Err(Error::NotAPsfFont);

            }

            let magic2 = [bytes[0], bytes[1], bytes[2], bytes[3]];

            if magic2 == [0x72, 0xb5, 0x4a, 0x86] {

                let header = &*(bytes.as_ptr() as *const Pfs2Header);

                let glyphs = &bytes[header.header_size as usize..];



                Ok(Font {

                    header: Version::V2(*header),

                    glyphs,

                })

            } else {

                Err(Error::InvalidMagic)

            }

        }

    }



    pub fn glyph(&self, c: char) -> Option<&'static [u8]> {

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

                    // Placeholder for unicode table lookup

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

}
