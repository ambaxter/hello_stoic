use lazy_static::lazy_static;

static ENCHIRIDION_STR: &'static str = include_str!("../static/enchiridion.txt");

pub fn extract_enchiridion() -> &'static [&'static str] {
    lazy_static! {
        static ref ENCHIRIDION: Vec<&'static str> = ENCHIRIDION_STR.split("****").collect();
    }
    &ENCHIRIDION
}
