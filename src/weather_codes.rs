#[must_use]
pub fn convert_code_to_image(code: i16) -> &'static [u8]{
    match code{
        2 => include_bytes!("assets/Sonne_mit_Wolke.bmp"),
        3 | 45 | 48 => include_bytes!("assets/Wolke.bmp"),
        60..=66 | 80..=85 => include_bytes!("assets/Regen.bmp"),
        67..=77 | 86 =>include_bytes!("assets/Schnee.bmp"),
        90..=99 =>include_bytes!("assets/Blitz.bmp"),
        _ => include_bytes!("assets/Sonne.bmp"),
 
    }
    
}
