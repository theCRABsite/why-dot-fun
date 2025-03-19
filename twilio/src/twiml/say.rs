use super::{format_xml_string, Action};

pub enum Voice {
    Man,
    Woman,
    Custom(String),
}

pub struct Say {
    pub txt: String,
    pub voice: Voice,
    pub language: String,
}

impl Action for Say {
    fn as_twiml(&self) -> String {
        let voice_str = match self.voice {
            Voice::Man => "man",
            Voice::Woman => "woman",
            Voice::Custom(ref s) => s.as_ref(),
        };
        format_xml_string(
            "Say",
            &vec![("voice", voice_str), ("language", &self.language)],
            &self.txt,
        )
    }
}
