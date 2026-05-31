use crate::keyword_map::keyword::Keyword;
use crate::language::Language;
use std::collections::HashMap;

pub struct KeywordMap {
    mappings: HashMap<Language, HashMap<String, Keyword>>,
}

impl KeywordMap {
    pub fn new() -> Self {
        let mut map = Self {
            mappings: HashMap::new(),
        };
        map.init_english();
        map.init_turkish();
        map.init_japanese();
        map.init_arabic();
        map.init_russian();
        map.init_chinese();
        map.init_spanish();
        map.init_german();
        map.init_french();
        map.init_italian();
        map.init_portuguese();
        map.init_polish();
        map.init_thai();
        map.init_indonesian();
        map.init_vietnamese();
        map.init_greek();
        map.init_serbian();
        map.init_bosnian();
        map.init_croatian();
        map.init_kurdish();
        map.init_persian();
        map.init_hindi();
        map.init_bengali();
        map.init_korean();
        map
    }

    pub fn lookup(&self, word: &str, language: Language) -> Option<Keyword> {
        self.mappings
            .get(&language)
            .and_then(|map| map.get(word))
            .copied()
    }

    fn init_english(&mut self) {
        use crate::languages::en;
        self.mappings.insert(Language::English, en::get_keywords());
    }
    fn init_turkish(&mut self) {
        use crate::languages::tr;
        self.mappings.insert(Language::Turkish, tr::get_keywords());
    }
    fn init_japanese(&mut self) {
        use crate::languages::ja;
        self.mappings.insert(Language::Japanese, ja::get_keywords());
    }
    fn init_arabic(&mut self) {
        use crate::languages::ar;
        self.mappings.insert(Language::Arabic, ar::get_keywords());
    }
    fn init_russian(&mut self) {
        use crate::languages::ru;
        self.mappings.insert(Language::Russian, ru::get_keywords());
    }
    fn init_chinese(&mut self) {
        use crate::languages::zh;
        self.mappings.insert(Language::Chinese, zh::get_keywords());
    }
    fn init_spanish(&mut self) {
        use crate::languages::es;
        self.mappings.insert(Language::Spanish, es::get_keywords());
    }
    fn init_german(&mut self) {
        use crate::languages::de;
        self.mappings.insert(Language::German, de::get_keywords());
    }
    fn init_french(&mut self) {
        use crate::languages::fr;
        self.mappings.insert(Language::French, fr::get_keywords());
    }
    fn init_italian(&mut self) {
        use crate::languages::it;
        self.mappings.insert(Language::Italian, it::get_keywords());
    }
    fn init_portuguese(&mut self) {
        use crate::languages::pt;
        self.mappings
            .insert(Language::Portuguese, pt::get_keywords());
    }
    fn init_polish(&mut self) {
        use crate::languages::pl;
        self.mappings.insert(Language::Polish, pl::get_keywords());
    }
    fn init_thai(&mut self) {
        use crate::languages::th;
        self.mappings.insert(Language::Thai, th::get_keywords());
    }
    fn init_indonesian(&mut self) {
        use crate::languages::id;
        self.mappings
            .insert(Language::Indonesian, id::get_keywords());
    }
    fn init_vietnamese(&mut self) {
        use crate::languages::vi;
        self.mappings
            .insert(Language::Vietnamese, vi::get_keywords());
    }
    fn init_greek(&mut self) {
        use crate::languages::el;
        self.mappings.insert(Language::Greek, el::get_keywords());
    }
    fn init_serbian(&mut self) {
        use crate::languages::sr;
        self.mappings.insert(Language::Serbian, sr::get_keywords());
    }
    fn init_bosnian(&mut self) {
        use crate::languages::bs;
        self.mappings.insert(Language::Bosnian, bs::get_keywords());
    }
    fn init_croatian(&mut self) {
        use crate::languages::hr;
        self.mappings.insert(Language::Croatian, hr::get_keywords());
    }
    fn init_kurdish(&mut self) {
        use crate::languages::ku;
        self.mappings.insert(Language::Kurdish, ku::get_keywords());
    }
    fn init_persian(&mut self) {
        use crate::languages::fa;
        self.mappings.insert(Language::Persian, fa::get_keywords());
    }
    fn init_hindi(&mut self) {
        use crate::languages::hi;
        self.mappings.insert(Language::Hindi, hi::get_keywords());
    }
    fn init_bengali(&mut self) {
        use crate::languages::bn;
        self.mappings.insert(Language::Bengali, bn::get_keywords());
    }
    fn init_korean(&mut self) {
        use crate::languages::ko;
        self.mappings.insert(Language::Korean, ko::get_keywords());
    }
}

impl Default for KeywordMap {
    fn default() -> Self {
        Self::new()
    }
}
