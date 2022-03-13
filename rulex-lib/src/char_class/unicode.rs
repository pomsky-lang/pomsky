use crate::error::CharClassError;

use super::char_group::GroupName;

pub(super) fn parse_group_name(name: &str) -> Result<GroupName, CharClassError> {
    match PARSE_LUT.binary_search_by_key(&name, |(k, _)| k) {
        Ok(n) => Ok(PARSE_LUT[n].1),
        Err(_) => Err(CharClassError::UnknownNamedClass(name.to_string())),
    }
}

// The following macro is used to generate the code below; however,
// the lookup table must be sorted manually to allow binary search.

/*
macro_rules! data {
    (
        $(
            $($name0:literal),* => $value0:expr;
        )*

        $(
            #$kind:ident( $prefix:literal ):
            $(
                $name:ident $(, $alternative:ident)* -> $default_repr:literal;
            )*
        )*
    ) => {
        $(
            #[derive(Clone, Copy, PartialEq, Eq)]
            #[allow(non_camel_case_types)]
            #[repr(u8)]
            #[rustfmt::skip]
            pub(crate) enum $kind {
                $( $name, )*
            }

            impl $kind {
                pub(super) fn as_str(self) -> &'static str {
                    static LUT: &[&str] = &[
                        $( $default_repr, )*
                    ];
                    LUT[self as u8 as usize]
                }
            }
        )*

        static PARSE_LUT: &[(&str, GroupName)] = &[
            $($(
                ( $name0, $value0 ),
            )*)*
            $(
                $(
                    ( concat!($prefix, stringify!($name)), GroupName::$kind($kind::$name) ),
                    $(
                        (concat!($prefix, stringify!($alternative)), GroupName::$kind($kind::$name)),
                    )*
                )*
            )*
        ];
    }
}

// https://tc39.es/ecma262/multipage/text-processing.html#table-unicode-script-values
data! {
    "word", "w" => GroupName::Word;
    "horiz_space", "h" => GroupName::HorizSpace;
    "vert_space", "v" => GroupName::VertSpace;
    "line_break", "l" => GroupName::LineBreak;

    #Category(""):

    Cased_Letter, LC -> "LC";
    Close_Punctuation, Pe -> "Pe";
    Connector_Punctuation, Pc -> "Pc";
    Control, Cc, cntrl -> "Cc";
    Currency_Symbol, Sc -> "Sc";
    Dash_Punctuation, Pd -> "Pd";
    Decimal_Number, Nd, digit, d -> "Nd";
    Enclosing_Mark, Me -> "Me";
    Final_Punctuation, Pf -> "Pf";
    Format, Cf -> "Cf";
    Initial_Punctuation, Pi -> "Pi";
    Letter, L -> "L";
    Letter_Number, Nl -> "Nl";
    Line_Separator, Zl -> "Zl";
    Lowercase_Letter, Ll  -> "Ll";
    Mark, M, Combining_Mark  -> "M";
    Math_Symbol, Sm  -> "Sm";
    Modifier_Letter, Lm  -> "Lm";
    Modifier_Symbol, Sk  -> "Sk";
    Nonspacing_Mark, Mn  -> "Mn";
    Number, N  -> "N";
    Open_Punctuation, Ps  -> "Ps";
    Other, C  -> "C";
    Other_Letter, Lo  -> "Lo";
    Other_Number, No  -> "No";
    Other_Punctuation, Po  -> "Po";
    Other_Symbol, So  -> "So";
    Paragraph_Separator, Zp  -> "Zp";
    Private_Use, Co  -> "Co";
    Punctuation, P, punct  -> "P";
    Separator, Z, space, s  -> "Z";
    Space_Separator, Zs  -> "Zs";
    Spacing_Mark, Mc  -> "Mc";
    Surrogate, Cs  -> "Cs";
    Symbol, S  -> "S";
    Titlecase_Letter, Lt  -> "Lt";
    Unassigned, Cn  -> "Cn";
    Uppercase_Letter, Lu -> "Lu";


    #Script(""):

    Adlam, Adlm -> "Adlam";
    Ahom -> "Ahom";
    Anatolian_Hieroglyphs, Hluw -> "Anatolian_Hieroglyphs";
    Arabic, Arab -> "Arabic";
    Armenian, Armn -> "Armenian";
    Avestan, Avst -> "Avestan";
    Balinese, Bali -> "Balinese";
    Bamum, Bamu -> "Bamum";
    Bassa_Vah, Bass -> "Bassa_Vah";
    Batak, Batk -> "Batak";
    Bengali, Beng -> "Bengali";
    Bhaiksuki, Bhks -> "Bhaiksuki";
    Bopomofo, Bopo -> "Bopomofo";
    Brahmi, Brah -> "Brahmi";
    Braille, Brai -> "Braille";
    Buginese, Bugi -> "Buginese";
    Buhid, Buhd -> "Buhid";
    Canadian_Aboriginal, Cans -> "Canadian_Aboriginal";
    Carian, Cari -> "Carian";
    Caucasian_Albanian, Aghb -> "Caucasian_Albanian";
    Chakma, Cakm -> "Chakma";
    Cham -> "Cham";
    Chorasmian, Chrs -> "Chorasmian";
    Cherokee, Cher -> "Cherokee";
    Common, Zyyy -> "Common";
    Coptic, Copt, Qaac -> "Coptic";
    Cuneiform, Xsux -> "Cuneiform";
    Cypriot, Cprt -> "Cypriot";
    Cypro_Minoan, Cpmn -> "Cypro_Minoan";
    Cyrillic, Cyrl -> "Cyrillic";
    Deseret, Dsrt -> "Deseret";
    Devanagari, Deva -> "Devanagari";
    Dives_Akuru, Diak -> "Dives_Akuru";
    Dogra, Dogr -> "Dogra";
    Duployan, Dupl -> "Duployan";
    Egyptian_Hieroglyphs, Egyp -> "Egyptian_Hieroglyphs";
    Elbasan, Elba -> "Elbasan";
    Elymaic, Elym -> "Elymaic";
    Ethiopic, Ethi -> "Ethiopic";
    Georgian, Geor -> "Georgian";
    Glagolitic, Glag -> "Glagolitic";
    Gothic, Goth -> "Gothic";
    Grantha, Gran -> "Grantha";
    Greek, Grek -> "Greek";
    Gujarati, Gujr -> "Gujarati";
    Gunjala_Gondi, Gong -> "Gunjala_Gondi";
    Gurmukhi, Guru -> "Gurmukhi";
    Han, Hani -> "Han";
    Hangul, Hang -> "Hangul";
    Hanifi_Rohingya, Rohg -> "Hanifi_Rohingya";
    Hanunoo, Hano -> "Hanunoo";
    Hatran, Hatr -> "Hatran";
    Hebrew, Hebr -> "Hebrew";
    Hiragana, Hira -> "Hiragana";
    Imperial_Aramaic, Armi -> "Imperial_Aramaic";
    Inherited, Zinh, Qaai -> "Inherited";
    Inscriptional_Pahlavi, Phli -> "Inscriptional_Pahlavi";
    Inscriptional_Parthian, Prti -> "Inscriptional_Parthian";
    Javanese, Java -> "Javanese";
    Kaithi, Kthi -> "Kaithi";
    Kannada, Knda -> "Kannada";
    Katakana, Kana -> "Katakana";
    Kayah_Li, Kali -> "Kayah_Li";
    Kharoshthi, Khar -> "Kharoshthi";
    Khitan_Small_Script, Kits -> "Khitan_Small_Script";
    Khmer, Khmr -> "Khmer";
    Khojki, Khoj -> "Khojki";
    Khudawadi, Sind -> "Khudawadi";
    Lao, Laoo -> "Lao";
    Latin, Latn -> "Latin";
    Lepcha, Lepc -> "Lepcha";
    Limbu, Limb -> "Limbu";
    Linear_A, Lina -> "Linear_A";
    Linear_B, Linb -> "Linear_B";
    Lisu -> "Lisu";
    Lycian, Lyci -> "Lycian";
    Lydian, Lydi -> "Lydian";
    Mahajani, Mahj -> "Mahajani";
    Makasar, Maka -> "Makasar";
    Malayalam, Mlym -> "Malayalam";
    Mandaic, Mand -> "Mandaic";
    Manichaean, Mani -> "Manichaean";
    Marchen, Marc -> "Marchen";
    Medefaidrin, Medf -> "Medefaidrin";
    Masaram_Gondi, Gonm -> "Masaram_Gondi";
    Meetei_Mayek, Mtei -> "Meetei_Mayek";
    Mende_Kikakui, Mend -> "Mende_Kikakui";
    Meroitic_Cursive, Merc -> "Meroitic_Cursive";
    Meroitic_Hieroglyphs, Mero -> "Meroitic_Hieroglyphs";
    Miao, Plrd -> "Miao";
    Modi -> "Modi";
    Mongolian, Mong -> "Mongolian";
    Mro, Mroo -> "Mro";
    Multani, Mult -> "Multani";
    Myanmar, Mymr -> "Myanmar";
    Nabataean, Nbat -> "Nabataean";
    Nandinagari, Nand -> "Nandinagari";
    New_Tai_Lue, Talu -> "New_Tai_Lue";
    Newa -> "Newa";
    Nko, Nkoo -> "Nko";
    Nushu, Nshu -> "Nushu";
    Nyiakeng_Puachue_Hmong, Hmnp -> "Nyiakeng_Puachue_Hmong";
    Ogham, Ogam -> "Ogham";
    Ol_Chiki, Olck -> "Ol_Chiki";
    Old_Hungarian, Hung -> "Old_Hungarian";
    Old_Italic, Ital -> "Old_Italic";
    Old_North_Arabian, Narb -> "Old_North_Arabian";
    Old_Permic, Perm -> "Old_Permic";
    Old_Persian, Xpeo -> "Old_Persian";
    Old_Sogdian, Sogo -> "Old_Sogdian";
    Old_South_Arabian, Sarb -> "Old_South_Arabian";
    Old_Turkic, Orkh -> "Old_Turkic";
    Old_Uyghur, Ougr -> "Old_Uyghur";
    Oriya, Orya -> "Oriya";
    Osage, Osge -> "Osage";
    Osmanya, Osma -> "Osmanya";
    Pahawh_Hmong, Hmng -> "Pahawh_Hmong";
    Palmyrene, Palm -> "Palmyrene";
    Pau_Cin_Hau, Pauc -> "Pau_Cin_Hau";
    Phags_Pa, Phag -> "Phags_Pa";
    Phoenician, Phnx -> "Phoenician";
    Psalter_Pahlavi, Phlp -> "Psalter_Pahlavi";
    Rejang, Rjng -> "Rejang";
    Runic, Runr -> "Runic";
    Samaritan, Samr -> "Samaritan";
    Saurashtra, Saur -> "Saurashtra";
    Sharada, Shrd -> "Sharada";
    Shavian, Shaw -> "Shavian";
    Siddham, Sidd -> "Siddham";
    SignWriting, Sgnw -> "SignWriting";
    Sinhala, Sinh -> "Sinhala";
    Sogdian, Sogd -> "Sogdian";
    Sora_Sompeng, Sora -> "Sora_Sompeng";
    Soyombo, Soyo -> "Soyombo";
    Sundanese, Sund -> "Sundanese";
    Syloti_Nagri, Sylo -> "Syloti_Nagri";
    Syriac, Syrc -> "Syriac";
    Tagalog, Tglg -> "Tagalog";
    Tagbanwa, Tagb -> "Tagbanwa";
    Tai_Le, Tale -> "Tai_Le";
    Tai_Tham, Lana -> "Tai_Tham";
    Tai_Viet, Tavt -> "Tai_Viet";
    Takri, Takr -> "Takri";
    Tamil, Taml -> "Tamil";
    Tangsa, Tnsa -> "Tangsa";
    Tangut, Tang -> "Tangut";
    Telugu, Telu -> "Telugu";
    Thaana, Thaa -> "Thaana";
    Thai -> "Thai";
    Tibetan, Tibt -> "Tibetan";
    Tifinagh, Tfng -> "Tifinagh";
    Tirhuta, Tirh -> "Tirhuta";
    Toto -> "Toto";
    Ugaritic, Ugar -> "Ugaritic";
    Vai, Vaii -> "Vai";
    Vithkuqi, Vith -> "Vithkuqi";
    Wancho, Wcho -> "Wancho";
    Warang_Citi, Wara -> "Warang_Citi";
    Yezidi, Yezi -> "Yezidi";
    Yi, Yiii -> "Yi";
    Zanabazar_Square, Zanb -> "Zanb";


    #CodeBlock("In"):

    Basic_Latin -> "Basic_Latin";
    Latin_1_Supplement -> "Latin-1_Supplement";
    Latin_Extended_A -> "Latin_Extended-A";
    Latin_Extended_B -> "Latin_Extended-B";
    IPA_Extensions -> "IPA_Extensions";
    Spacing_Modifier_Letters -> "Spacing_Modifier_Letters";
    Combining_Diacritical_Marks -> "Combining_Diacritical_Marks";
    Greek_and_Coptic -> "Greek_and_Coptic";
    Cyrillic -> "Cyrillic";
    Cyrillic_Supplementary -> "Cyrillic_Supplementary";
    Armenian -> "Armenian";
    Hebrew -> "Hebrew";
    Arabic -> "Arabic";
    Syriac -> "Syriac";
    Thaana -> "Thaana";
    Devanagari -> "Devanagari";
    Bengali -> "Bengali";
    Gurmukhi -> "Gurmukhi";
    Gujarati -> "Gujarati";
    Oriya -> "Oriya";
    Tamil -> "Tamil";
    Telugu -> "Telugu";
    Kannada -> "Kannada";
    Malayalam -> "Malayalam";
    Sinhala -> "Sinhala";
    Thai -> "Thai";
    Lao -> "Lao";
    Tibetan -> "Tibetan";
    Myanmar -> "Myanmar";
    Georgian -> "Georgian";
    Hangul_Jamo -> "Hangul_Jamo";
    Ethiopic -> "Ethiopic";
    Cherokee -> "Cherokee";
    Unified_Canadian_Aboriginal_Syllabics -> "Unified_Canadian_Aboriginal_Syllabics";
    Ogham -> "Ogham";
    Runic -> "Runic";
    Tagalog -> "Tagalog";
    Hanunoo -> "Hanunoo";
    Buhid -> "Buhid";
    Tagbanwa -> "Tagbanwa";
    Khmer -> "Khmer";
    Mongolian -> "Mongolian";
    Limbu -> "Limbu";
    Tai_Le -> "Tai_Le";
    Khmer_Symbols -> "Khmer_Symbols";
    Phonetic_Extensions -> "Phonetic_Extensions";
    Latin_Extended_Additional -> "Latin_Extended_Additional";
    Greek_Extended -> "Greek_Extended";
    General_Punctuation -> "General_Punctuation";
    Superscripts_and_Subscripts -> "Superscripts_and_Subscripts";
    Currency_Symbols -> "Currency_Symbols";
    Combining_Diacritical_Marks_for_Symbols -> "Combining_Diacritical_Marks_for_Symbols";
    Letterlike_Symbols -> "Letterlike_Symbols";
    Number_Forms -> "Number_Forms";
    Arrows -> "Arrows";
    Mathematical_Operators -> "Mathematical_Operators";
    Miscellaneous_Technical -> "Miscellaneous_Technical";
    Control_Pictures -> "Control_Pictures";
    Optical_Character_Recognition -> "Optical_Character_Recognition";
    Enclosed_Alphanumerics -> "Enclosed_Alphanumerics";
    Box_Drawing -> "Box_Drawing";
    Block_Elements -> "Block_Elements";
    Geometric_Shapes -> "Geometric_Shapes";
    Miscellaneous_Symbols -> "Miscellaneous_Symbols";
    Dingbats -> "Dingbats";
    Miscellaneous_Mathematical_Symbols_A -> "Miscellaneous_Mathematical_Symbols-A";
    Supplemental_Arrows_A -> "Supplemental_Arrows-A";
    Braille_Patterns -> "Braille_Patterns";
    Supplemental_Arrows_B -> "Supplemental_Arrows-B";
    Miscellaneous_Mathematical_Symbols_B -> "Miscellaneous_Mathematical_Symbols-B";
    Supplemental_Mathematical_Operators -> "Supplemental_Mathematical_Operators";
    Miscellaneous_Symbols_and_Arrows -> "Miscellaneous_Symbols_and_Arrows";
    CJK_Radicals_Supplement -> "CJK_Radicals_Supplement";
    Kangxi_Radicals -> "Kangxi_Radicals";
    Ideographic_Description_Characters -> "Ideographic_Description_Characters";
    CJK_Symbols_and_Punctuation -> "CJK_Symbols_and_Punctuation";
    Hiragana -> "Hiragana";
    Katakana -> "Katakana";
    Bopomofo -> "Bopomofo";
    Hangul_Compatibility_Jamo -> "Hangul_Compatibility_Jamo";
    Kanbun -> "Kanbun";
    Bopomofo_Extended -> "Bopomofo_Extended";
    Katakana_Phonetic_Extensions -> "Katakana_Phonetic_Extensions";
    Enclosed_CJK_Letters_and_Months -> "Enclosed_CJK_Letters_and_Months";
    CJK_Compatibility -> "CJK_Compatibility";
    CJK_Unified_Ideographs_Extension_A -> "CJK_Unified_Ideographs_Extension_A";
    Yijing_Hexagram_Symbols -> "Yijing_Hexagram_Symbols";
    CJK_Unified_Ideographs -> "CJK_Unified_Ideographs";
    Yi_Syllables -> "Yi_Syllables";
    Yi_Radicals -> "Yi_Radicals";
    Hangul_Syllables -> "Hangul_Syllables";
    High_Surrogates -> "High_Surrogates";
    High_Private_Use_Surrogates -> "High_Private_Use_Surrogates";
    Low_Surrogates -> "Low_Surrogates";
    Private_Use_Area -> "Private_Use_Area";
    CJK_Compatibility_Ideographs -> "CJK_Compatibility_Ideographs";
    Alphabetic_Presentation_Forms -> "Alphabetic_Presentation_Forms";
    Arabic_Presentation_Forms_A -> "Arabic_Presentation_Forms-A";
    Variation_Selectors -> "Variation_Selectors";
    Combining_Half_Marks -> "Combining_Half_Marks";
    CJK_Compatibility_Forms -> "CJK_Compatibility_Forms";
    Small_Form_Variants -> "Small_Form_Variants";
    Arabic_Presentation_Forms_B -> "Arabic_Presentation_Forms-B";
    Halfwidth_and_Fullwidth_Forms -> "Halfwidth_and_Fullwidth_Forms";
    Specials -> "Specials";

    // https://unicode.org/reports/tr44/#Property_Index
    #OtherProperties(""):

    White_Space -> "White_Space";
    Alphabetic, alpha -> "Alphabetic";
    Noncharacter_Code_Point -> "Noncharacter_Code_Point";
    Default_Ignorable_Code_Point -> "Default_Ignorable_Code_Point";
    Logical_Order_Exception -> "Logical_Order_Exception";
    Deprecated -> "Deprecated";
    Variation_Selector -> "Variation_Selector";

    Uppercase, upper -> "Uppercase";
    Lowercase, lower -> "Lowercase";
    Soft_Dotted -> "Soft_Dotted";
    Case_Ignorable -> "Case_Ignorable";
    Changes_When_Lowercased -> "Changes_When_Lowercased";
    Changes_When_Uppercased -> "Changes_When_Uppercased";
    Changes_When_Titlecased -> "Changes_When_Titlecased";
    Changes_When_Casefolded -> "Changes_When_Casefolded";
    Changes_When_Casemapped -> "Changes_When_Casemapped";

    Emoji -> "Emoji";
    Emoji_Presentation -> "Emoji_Presentation";
    Emoji_Modifier -> "Emoji_Modifier";
    Emoji_Modifier_Base -> "Emoji_Modifier_Base";
    Emoji_Component -> "Emoji_Component";
    Extended_Pictographic -> "Extended_Pictographic";

    Hex_Digit -> "Hex_Digit";
    ASCII_Hex_Digit -> "ASCII_Hex_Digit";

    Join_Control -> "Join_Control";
    Joining_Group -> "Joining_Group";

    Bidi_Control -> "Bidi_Control";
    Bidi_Mirrored -> "Bidi_Mirrored";
    Bidi_Mirroring_Glyph -> "Bidi_Mirroring_Glyph";

    ID_Continue -> "ID_Continue";
    ID_Start -> "ID_Start";
    XID_Continue -> "XID_Continue";
    XID_Start -> "XID_Start";
    Pattern_Syntax -> "Pattern_Syntax";
    Pattern_White_Space -> "Pattern_White_Space";

    Ideographic -> "Ideographic";
    Unified_Ideograph -> "Unified_Ideograph";
    Radical -> "Radical";
    IDS_Binary_Operator -> "IDS_Binary_Operator";
    IDS_Trinary_Operator -> "IDS_Trinary_Operator";

    Math -> "Math";
    Quotation_Mark -> "Quotation_Mark";
    Dash -> "Dash";
    Sentence_Terminal -> "Sentence_Terminal";
    Terminal_Punctuation -> "Terminal_Punctuation";
    Diacritic -> "Diacritic";
    Extender -> "Extender";
    Grapheme_Base -> "Grapheme_Base";
    Grapheme_Extend -> "Grapheme_Extend";
    Regional_Indicator -> "Regional_Indicator";
}
*/

// Recursive expansion of data! macro
// ===================================

#[derive(Clone, Copy, PartialEq, Eq)]
#[allow(non_camel_case_types)]
#[repr(u8)]
#[rustfmt::skip]
pub(crate) enum Category {
    Cased_Letter, Close_Punctuation, Connector_Punctuation, Control, Currency_Symbol,
    Dash_Punctuation, Decimal_Number, Enclosing_Mark, Final_Punctuation, Format,
    Initial_Punctuation, Letter, Letter_Number, Line_Separator, Lowercase_Letter, Mark, Math_Symbol,
    Modifier_Letter, Modifier_Symbol, Nonspacing_Mark, Number, Open_Punctuation, Other,
    Other_Letter, Other_Number, Other_Punctuation, Other_Symbol, Paragraph_Separator, Private_Use,
    Punctuation, Separator, Space_Separator, Spacing_Mark, Surrogate, Symbol, Titlecase_Letter,
    Unassigned, Uppercase_Letter, 
}

impl Category {
    pub(super) fn as_str(self) -> &'static str {
        #[rustfmt::skip]
        static LUT: &[&str] = &[
            "LC", "Pe", "Pc", "Cc", "Sc", "Pd", "Nd", "Me", "Pf", "Cf", "Pi", "L", "Nl", "Zl",
            "Ll", "M", "Sm", "Lm", "Sk", "Mn", "N", "Ps", "C", "Lo", "No", "Po", "So", "Zp", "Co",
            "P", "Z", "Zs", "Mc", "Cs", "S", "Lt", "Cn", "Lu",
        ];
        LUT[self as u8 as usize]
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[allow(non_camel_case_types)]
#[repr(u8)]
#[rustfmt::skip]
pub(crate) enum Script {
    Adlam, Ahom, Anatolian_Hieroglyphs, Arabic, Armenian, Avestan, Balinese, Bamum, Bassa_Vah,
    Batak, Bengali, Bhaiksuki, Bopomofo, Brahmi, Braille, Buginese, Buhid, Canadian_Aboriginal,
    Carian, Caucasian_Albanian, Chakma, Cham, Chorasmian, Cherokee, Common, Coptic, Cuneiform,
    Cypriot, Cypro_Minoan, Cyrillic, Deseret, Devanagari, Dives_Akuru, Dogra, Duployan,
    Egyptian_Hieroglyphs, Elbasan, Elymaic, Ethiopic, Georgian, Glagolitic, Gothic, Grantha, Greek,
    Gujarati, Gunjala_Gondi, Gurmukhi, Han, Hangul, Hanifi_Rohingya, Hanunoo, Hatran, Hebrew,
    Hiragana, Imperial_Aramaic, Inherited, Inscriptional_Pahlavi, Inscriptional_Parthian, Javanese,
    Kaithi, Kannada, Katakana, Kayah_Li, Kharoshthi, Khitan_Small_Script, Khmer, Khojki, Khudawadi,
    Lao, Latin, Lepcha, Limbu, Linear_A, Linear_B, Lisu, Lycian, Lydian, Mahajani, Makasar,
    Malayalam, Mandaic, Manichaean, Marchen, Medefaidrin, Masaram_Gondi, Meetei_Mayek,
    Mende_Kikakui, Meroitic_Cursive, Meroitic_Hieroglyphs, Miao, Modi, Mongolian, Mro, Multani,
    Myanmar, Nabataean, Nandinagari, New_Tai_Lue, Newa, Nko, Nushu, Nyiakeng_Puachue_Hmong, Ogham,
    Ol_Chiki, Old_Hungarian, Old_Italic, Old_North_Arabian, Old_Permic, Old_Persian, Old_Sogdian,
    Old_South_Arabian, Old_Turkic, Old_Uyghur, Oriya, Osage, Osmanya, Pahawh_Hmong, Palmyrene,
    Pau_Cin_Hau, Phags_Pa, Phoenician, Psalter_Pahlavi, Rejang, Runic, Samaritan, Saurashtra,
    Sharada, Shavian, Siddham, SignWriting, Sinhala, Sogdian, Sora_Sompeng, Soyombo, Sundanese,
    Syloti_Nagri, Syriac, Tagalog, Tagbanwa, Tai_Le, Tai_Tham, Tai_Viet, Takri, Tamil, Tangsa,
    Tangut, Telugu, Thaana, Thai, Tibetan, Tifinagh, Tirhuta, Toto, Ugaritic, Vai, Vithkuqi, Wancho,
    Warang_Citi, Yezidi, Yi, Zanabazar_Square,
}

impl Script {
    pub(super) fn as_str(self) -> &'static str {
        #[rustfmt::skip]
        static LUT: &[&str] = &[
            "Adlam", "Ahom", "Anatolian_Hieroglyphs", "Arabic", "Armenian", "Avestan", "Balinese",
            "Bamum", "Bassa_Vah", "Batak", "Bengali", "Bhaiksuki", "Bopomofo", "Brahmi", "Braille",
            "Buginese", "Buhid", "Canadian_Aboriginal", "Carian", "Caucasian_Albanian", "Chakma",
            "Cham", "Chorasmian", "Cherokee", "Common", "Coptic", "Cuneiform", "Cypriot",
            "Cypro_Minoan", "Cyrillic", "Deseret", "Devanagari", "Dives_Akuru", "Dogra", "Duployan",
            "Egyptian_Hieroglyphs", "Elbasan", "Elymaic", "Ethiopic", "Georgian", "Glagolitic",
            "Gothic", "Grantha", "Greek", "Gujarati", "Gunjala_Gondi", "Gurmukhi", "Han", "Hangul",
            "Hanifi_Rohingya", "Hanunoo", "Hatran", "Hebrew", "Hiragana", "Imperial_Aramaic",
            "Inherited", "Inscriptional_Pahlavi", "Inscriptional_Parthian", "Javanese", "Kaithi",
            "Kannada", "Katakana", "Kayah_Li", "Kharoshthi", "Khitan_Small_Script", "Khmer",
            "Khojki", "Khudawadi", "Lao", "Latin", "Lepcha", "Limbu", "Linear_A", "Linear_B",
            "Lisu", "Lycian", "Lydian", "Mahajani", "Makasar", "Malayalam", "Mandaic", "Manichaean",
            "Marchen", "Medefaidrin", "Masaram_Gondi", "Meetei_Mayek", "Mende_Kikakui",
            "Meroitic_Cursive", "Meroitic_Hieroglyphs", "Miao", "Modi", "Mongolian", "Mro",
            "Multani", "Myanmar", "Nabataean", "Nandinagari", "New_Tai_Lue", "Newa", "Nko", "Nushu",
            "Nyiakeng_Puachue_Hmong", "Ogham", "Ol_Chiki", "Old_Hungarian", "Old_Italic",
            "Old_North_Arabian", "Old_Permic", "Old_Persian", "Old_Sogdian", "Old_South_Arabian",
            "Old_Turkic", "Old_Uyghur", "Oriya", "Osage", "Osmanya", "Pahawh_Hmong", "Palmyrene",
            "Pau_Cin_Hau", "Phags_Pa", "Phoenician", "Psalter_Pahlavi", "Rejang", "Runic",
            "Samaritan", "Saurashtra", "Sharada", "Shavian", "Siddham", "SignWriting", "Sinhala",
            "Sogdian", "Sora_Sompeng", "Soyombo", "Sundanese", "Syloti_Nagri", "Syriac", "Tagalog",
            "Tagbanwa", "Tai_Le", "Tai_Tham", "Tai_Viet", "Takri", "Tamil", "Tangsa", "Tangut",
            "Telugu", "Thaana", "Thai", "Tibetan", "Tifinagh", "Tirhuta", "Toto", "Ugaritic", "Vai",
            "Vithkuqi", "Wancho", "Warang_Citi", "Yezidi", "Yi", "Zanb",
        ];
        LUT[self as u8 as usize]
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[allow(non_camel_case_types)]
#[repr(u8)]
#[rustfmt::skip]
pub(crate) enum CodeBlock {
    Basic_Latin, Latin_1_Supplement, Latin_Extended_A, Latin_Extended_B, IPA_Extensions,
    Spacing_Modifier_Letters, Combining_Diacritical_Marks, Greek_and_Coptic, Cyrillic,
    Cyrillic_Supplementary, Armenian, Hebrew, Arabic, Syriac, Thaana, Devanagari, Bengali, Gurmukhi,
    Gujarati, Oriya, Tamil, Telugu, Kannada, Malayalam, Sinhala, Thai, Lao, Tibetan, Myanmar,
    Georgian, Hangul_Jamo, Ethiopic, Cherokee, Unified_Canadian_Aboriginal_Syllabics, Ogham, Runic,
    Tagalog, Hanunoo, Buhid, Tagbanwa, Khmer, Mongolian, Limbu, Tai_Le, Khmer_Symbols,
    Phonetic_Extensions, Latin_Extended_Additional, Greek_Extended, General_Punctuation,
    Superscripts_and_Subscripts, Currency_Symbols, Combining_Diacritical_Marks_for_Symbols,
    Letterlike_Symbols, Number_Forms, Arrows, Mathematical_Operators, Miscellaneous_Technical,
    Control_Pictures, Optical_Character_Recognition, Enclosed_Alphanumerics, Box_Drawing,
    Block_Elements, Geometric_Shapes, Miscellaneous_Symbols, Dingbats,
    Miscellaneous_Mathematical_Symbols_A, Supplemental_Arrows_A, Braille_Patterns,
    Supplemental_Arrows_B, Miscellaneous_Mathematical_Symbols_B,
    Supplemental_Mathematical_Operators, Miscellaneous_Symbols_and_Arrows, CJK_Radicals_Supplement,
    Kangxi_Radicals, Ideographic_Description_Characters, CJK_Symbols_and_Punctuation, Hiragana,
    Katakana, Bopomofo, Hangul_Compatibility_Jamo, Kanbun, Bopomofo_Extended,
    Katakana_Phonetic_Extensions, Enclosed_CJK_Letters_and_Months, CJK_Compatibility,
    CJK_Unified_Ideographs_Extension_A, Yijing_Hexagram_Symbols, CJK_Unified_Ideographs,
    Yi_Syllables, Yi_Radicals, Hangul_Syllables, High_Surrogates, High_Private_Use_Surrogates,
    Low_Surrogates, Private_Use_Area, CJK_Compatibility_Ideographs, Alphabetic_Presentation_Forms,
    Arabic_Presentation_Forms_A, Variation_Selectors, Combining_Half_Marks, CJK_Compatibility_Forms,
    Small_Form_Variants, Arabic_Presentation_Forms_B, Halfwidth_and_Fullwidth_Forms, Specials,
}

impl CodeBlock {
    pub(super) fn as_str(self) -> &'static str {
        #[rustfmt::skip]
        static LUT: &[&str] = &[
            "Basic_Latin", "Latin-1_Supplement", "Latin_Extended-A", "Latin_Extended-B",
            "IPA_Extensions", "Spacing_Modifier_Letters", "Combining_Diacritical_Marks",
            "Greek_and_Coptic", "Cyrillic", "Cyrillic_Supplementary", "Armenian", "Hebrew",
            "Arabic", "Syriac", "Thaana", "Devanagari", "Bengali", "Gurmukhi", "Gujarati", "Oriya",
            "Tamil", "Telugu", "Kannada", "Malayalam", "Sinhala", "Thai", "Lao", "Tibetan",
            "Myanmar", "Georgian", "Hangul_Jamo", "Ethiopic", "Cherokee",
            "Unified_Canadian_Aboriginal_Syllabics", "Ogham", "Runic", "Tagalog", "Hanunoo",
            "Buhid", "Tagbanwa", "Khmer", "Mongolian", "Limbu", "Tai_Le", "Khmer_Symbols",
            "Phonetic_Extensions", "Latin_Extended_Additional", "Greek_Extended",
            "General_Punctuation", "Superscripts_and_Subscripts", "Currency_Symbols",
            "Combining_Diacritical_Marks_for_Symbols", "Letterlike_Symbols", "Number_Forms",
            "Arrows", "Mathematical_Operators", "Miscellaneous_Technical", "Control_Pictures",
            "Optical_Character_Recognition", "Enclosed_Alphanumerics", "Box_Drawing",
            "Block_Elements", "Geometric_Shapes", "Miscellaneous_Symbols", "Dingbats",
            "Miscellaneous_Mathematical_Symbols-A", "Supplemental_Arrows-A", "Braille_Patterns",
            "Supplemental_Arrows-B", "Miscellaneous_Mathematical_Symbols-B",
            "Supplemental_Mathematical_Operators", "Miscellaneous_Symbols_and_Arrows",
            "CJK_Radicals_Supplement", "Kangxi_Radicals", "Ideographic_Description_Characters",
            "CJK_Symbols_and_Punctuation", "Hiragana", "Katakana", "Bopomofo",
            "Hangul_Compatibility_Jamo", "Kanbun", "Bopomofo_Extended",
            "Katakana_Phonetic_Extensions", "Enclosed_CJK_Letters_and_Months", "CJK_Compatibility",
            "CJK_Unified_Ideographs_Extension_A", "Yijing_Hexagram_Symbols",
            "CJK_Unified_Ideographs", "Yi_Syllables", "Yi_Radicals", "Hangul_Syllables",
            "High_Surrogates", "High_Private_Use_Surrogates", "Low_Surrogates", "Private_Use_Area",
            "CJK_Compatibility_Ideographs", "Alphabetic_Presentation_Forms",
            "Arabic_Presentation_Forms-A", "Variation_Selectors", "Combining_Half_Marks",
            "CJK_Compatibility_Forms", "Small_Form_Variants", "Arabic_Presentation_Forms-B",
            "Halfwidth_and_Fullwidth_Forms", "Specials",
        ];
        LUT[self as u8 as usize]
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[allow(non_camel_case_types)]
#[repr(u8)]
#[rustfmt::skip]
pub(crate) enum OtherProperties {
    White_Space, Alphabetic, Noncharacter_Code_Point, Default_Ignorable_Code_Point,
    Logical_Order_Exception, Deprecated, Variation_Selector, Uppercase, Lowercase, Soft_Dotted,
    Case_Ignorable, Changes_When_Lowercased, Changes_When_Uppercased, Changes_When_Titlecased,
    Changes_When_Casefolded, Changes_When_Casemapped, Emoji, Emoji_Presentation, Emoji_Modifier,
    Emoji_Modifier_Base, Emoji_Component, Extended_Pictographic, Hex_Digit, ASCII_Hex_Digit,
    Join_Control, Joining_Group, Bidi_Control, Bidi_Mirrored, Bidi_Mirroring_Glyph, ID_Continue,
    ID_Start, XID_Continue, XID_Start, Pattern_Syntax, Pattern_White_Space, Ideographic,
    Unified_Ideograph, Radical, IDS_Binary_Operator, IDS_Trinary_Operator, Math, Quotation_Mark,
    Dash, Sentence_Terminal, Terminal_Punctuation, Diacritic, Extender, Grapheme_Base,
    Grapheme_Extend, Regional_Indicator,
}

impl OtherProperties {
    pub(super) fn as_str(self) -> &'static str {
        #[rustfmt::skip]
        static LUT: &[&str] = &[
            "White_Space", "Alphabetic", "Noncharacter_Code_Point", "Default_Ignorable_Code_Point",
            "Logical_Order_Exception", "Deprecated", "Variation_Selector", "Uppercase", "Lowercase",
            "Soft_Dotted", "Case_Ignorable", "Changes_When_Lowercased", "Changes_When_Uppercased",
            "Changes_When_Titlecased", "Changes_When_Casefolded", "Changes_When_Casemapped",
            "Emoji", "Emoji_Presentation", "Emoji_Modifier", "Emoji_Modifier_Base",
            "Emoji_Component", "Extended_Pictographic", "Hex_Digit", "ASCII_Hex_Digit",
            "Join_Control", "Joining_Group", "Bidi_Control", "Bidi_Mirrored",
            "Bidi_Mirroring_Glyph", "ID_Continue", "ID_Start", "XID_Continue", "XID_Start",
            "Pattern_Syntax", "Pattern_White_Space", "Ideographic", "Unified_Ideograph", "Radical",
            "IDS_Binary_Operator", "IDS_Trinary_Operator", "Math", "Quotation_Mark", "Dash",
            "Sentence_Terminal", "Terminal_Punctuation", "Diacritic", "Extender", "Grapheme_Base",
            "Grapheme_Extend", "Regional_Indicator",
        ];
        LUT[self as u8 as usize]
    }
}

static PARSE_LUT: &[(&str, GroupName)] = &[
    ("ASCII_Hex_Digit", GroupName::OtherProperties(OtherProperties::ASCII_Hex_Digit)),
    ("Adlam", GroupName::Script(Script::Adlam)),
    ("Adlm", GroupName::Script(Script::Adlam)),
    ("Aghb", GroupName::Script(Script::Caucasian_Albanian)),
    ("Ahom", GroupName::Script(Script::Ahom)),
    ("Alphabetic", GroupName::OtherProperties(OtherProperties::Alphabetic)),
    ("Anatolian_Hieroglyphs", GroupName::Script(Script::Anatolian_Hieroglyphs)),
    ("Arab", GroupName::Script(Script::Arabic)),
    ("Arabic", GroupName::Script(Script::Arabic)),
    ("Armenian", GroupName::Script(Script::Armenian)),
    ("Armi", GroupName::Script(Script::Imperial_Aramaic)),
    ("Armn", GroupName::Script(Script::Armenian)),
    ("Avestan", GroupName::Script(Script::Avestan)),
    ("Avst", GroupName::Script(Script::Avestan)),
    ("Bali", GroupName::Script(Script::Balinese)),
    ("Balinese", GroupName::Script(Script::Balinese)),
    ("Bamu", GroupName::Script(Script::Bamum)),
    ("Bamum", GroupName::Script(Script::Bamum)),
    ("Bass", GroupName::Script(Script::Bassa_Vah)),
    ("Bassa_Vah", GroupName::Script(Script::Bassa_Vah)),
    ("Batak", GroupName::Script(Script::Batak)),
    ("Batk", GroupName::Script(Script::Batak)),
    ("Beng", GroupName::Script(Script::Bengali)),
    ("Bengali", GroupName::Script(Script::Bengali)),
    ("Bhaiksuki", GroupName::Script(Script::Bhaiksuki)),
    ("Bhks", GroupName::Script(Script::Bhaiksuki)),
    ("Bidi_Control", GroupName::OtherProperties(OtherProperties::Bidi_Control)),
    ("Bidi_Mirrored", GroupName::OtherProperties(OtherProperties::Bidi_Mirrored)),
    ("Bidi_Mirroring_Glyph", GroupName::OtherProperties(OtherProperties::Bidi_Mirroring_Glyph)),
    ("Bopo", GroupName::Script(Script::Bopomofo)),
    ("Bopomofo", GroupName::Script(Script::Bopomofo)),
    ("Brah", GroupName::Script(Script::Brahmi)),
    ("Brahmi", GroupName::Script(Script::Brahmi)),
    ("Brai", GroupName::Script(Script::Braille)),
    ("Braille", GroupName::Script(Script::Braille)),
    ("Bugi", GroupName::Script(Script::Buginese)),
    ("Buginese", GroupName::Script(Script::Buginese)),
    ("Buhd", GroupName::Script(Script::Buhid)),
    ("Buhid", GroupName::Script(Script::Buhid)),
    ("C", GroupName::Category(Category::Other)),
    ("Cakm", GroupName::Script(Script::Chakma)),
    ("Canadian_Aboriginal", GroupName::Script(Script::Canadian_Aboriginal)),
    ("Cans", GroupName::Script(Script::Canadian_Aboriginal)),
    ("Cari", GroupName::Script(Script::Carian)),
    ("Carian", GroupName::Script(Script::Carian)),
    ("Case_Ignorable", GroupName::OtherProperties(OtherProperties::Case_Ignorable)),
    ("Cased_Letter", GroupName::Category(Category::Cased_Letter)),
    ("Caucasian_Albanian", GroupName::Script(Script::Caucasian_Albanian)),
    ("Cc", GroupName::Category(Category::Control)),
    ("Cf", GroupName::Category(Category::Format)),
    ("Chakma", GroupName::Script(Script::Chakma)),
    ("Cham", GroupName::Script(Script::Cham)),
    (
        "Changes_When_Casefolded",
        GroupName::OtherProperties(OtherProperties::Changes_When_Casefolded),
    ),
    (
        "Changes_When_Casemapped",
        GroupName::OtherProperties(OtherProperties::Changes_When_Casemapped),
    ),
    (
        "Changes_When_Lowercased",
        GroupName::OtherProperties(OtherProperties::Changes_When_Lowercased),
    ),
    (
        "Changes_When_Titlecased",
        GroupName::OtherProperties(OtherProperties::Changes_When_Titlecased),
    ),
    (
        "Changes_When_Uppercased",
        GroupName::OtherProperties(OtherProperties::Changes_When_Uppercased),
    ),
    ("Cher", GroupName::Script(Script::Cherokee)),
    ("Cherokee", GroupName::Script(Script::Cherokee)),
    ("Chorasmian", GroupName::Script(Script::Chorasmian)),
    ("Chrs", GroupName::Script(Script::Chorasmian)),
    ("Close_Punctuation", GroupName::Category(Category::Close_Punctuation)),
    ("Cn", GroupName::Category(Category::Unassigned)),
    ("Co", GroupName::Category(Category::Private_Use)),
    ("Combining_Mark", GroupName::Category(Category::Mark)),
    ("Common", GroupName::Script(Script::Common)),
    ("Connector_Punctuation", GroupName::Category(Category::Connector_Punctuation)),
    ("Control", GroupName::Category(Category::Control)),
    ("Copt", GroupName::Script(Script::Coptic)),
    ("Coptic", GroupName::Script(Script::Coptic)),
    ("Cpmn", GroupName::Script(Script::Cypro_Minoan)),
    ("Cprt", GroupName::Script(Script::Cypriot)),
    ("Cs", GroupName::Category(Category::Surrogate)),
    ("Cuneiform", GroupName::Script(Script::Cuneiform)),
    ("Currency_Symbol", GroupName::Category(Category::Currency_Symbol)),
    ("Cypriot", GroupName::Script(Script::Cypriot)),
    ("Cypro_Minoan", GroupName::Script(Script::Cypro_Minoan)),
    ("Cyrillic", GroupName::Script(Script::Cyrillic)),
    ("Cyrl", GroupName::Script(Script::Cyrillic)),
    ("Dash", GroupName::OtherProperties(OtherProperties::Dash)),
    ("Dash_Punctuation", GroupName::Category(Category::Dash_Punctuation)),
    ("Decimal_Number", GroupName::Category(Category::Decimal_Number)),
    (
        "Default_Ignorable_Code_Point",
        GroupName::OtherProperties(OtherProperties::Default_Ignorable_Code_Point),
    ),
    ("Deprecated", GroupName::OtherProperties(OtherProperties::Deprecated)),
    ("Deseret", GroupName::Script(Script::Deseret)),
    ("Deva", GroupName::Script(Script::Devanagari)),
    ("Devanagari", GroupName::Script(Script::Devanagari)),
    ("Diacritic", GroupName::OtherProperties(OtherProperties::Diacritic)),
    ("Diak", GroupName::Script(Script::Dives_Akuru)),
    ("Dives_Akuru", GroupName::Script(Script::Dives_Akuru)),
    ("Dogr", GroupName::Script(Script::Dogra)),
    ("Dogra", GroupName::Script(Script::Dogra)),
    ("Dsrt", GroupName::Script(Script::Deseret)),
    ("Dupl", GroupName::Script(Script::Duployan)),
    ("Duployan", GroupName::Script(Script::Duployan)),
    ("Egyp", GroupName::Script(Script::Egyptian_Hieroglyphs)),
    ("Egyptian_Hieroglyphs", GroupName::Script(Script::Egyptian_Hieroglyphs)),
    ("Elba", GroupName::Script(Script::Elbasan)),
    ("Elbasan", GroupName::Script(Script::Elbasan)),
    ("Elym", GroupName::Script(Script::Elymaic)),
    ("Elymaic", GroupName::Script(Script::Elymaic)),
    ("Emoji", GroupName::OtherProperties(OtherProperties::Emoji)),
    ("Emoji_Component", GroupName::OtherProperties(OtherProperties::Emoji_Component)),
    ("Emoji_Modifier", GroupName::OtherProperties(OtherProperties::Emoji_Modifier)),
    ("Emoji_Modifier_Base", GroupName::OtherProperties(OtherProperties::Emoji_Modifier_Base)),
    ("Emoji_Presentation", GroupName::OtherProperties(OtherProperties::Emoji_Presentation)),
    ("Enclosing_Mark", GroupName::Category(Category::Enclosing_Mark)),
    ("Ethi", GroupName::Script(Script::Ethiopic)),
    ("Ethiopic", GroupName::Script(Script::Ethiopic)),
    ("Extended_Pictographic", GroupName::OtherProperties(OtherProperties::Extended_Pictographic)),
    ("Extender", GroupName::OtherProperties(OtherProperties::Extender)),
    ("Final_Punctuation", GroupName::Category(Category::Final_Punctuation)),
    ("Format", GroupName::Category(Category::Format)),
    ("Geor", GroupName::Script(Script::Georgian)),
    ("Georgian", GroupName::Script(Script::Georgian)),
    ("Glag", GroupName::Script(Script::Glagolitic)),
    ("Glagolitic", GroupName::Script(Script::Glagolitic)),
    ("Gong", GroupName::Script(Script::Gunjala_Gondi)),
    ("Gonm", GroupName::Script(Script::Masaram_Gondi)),
    ("Goth", GroupName::Script(Script::Gothic)),
    ("Gothic", GroupName::Script(Script::Gothic)),
    ("Gran", GroupName::Script(Script::Grantha)),
    ("Grantha", GroupName::Script(Script::Grantha)),
    ("Grapheme_Base", GroupName::OtherProperties(OtherProperties::Grapheme_Base)),
    ("Grapheme_Extend", GroupName::OtherProperties(OtherProperties::Grapheme_Extend)),
    ("Greek", GroupName::Script(Script::Greek)),
    ("Grek", GroupName::Script(Script::Greek)),
    ("Gujarati", GroupName::Script(Script::Gujarati)),
    ("Gujr", GroupName::Script(Script::Gujarati)),
    ("Gunjala_Gondi", GroupName::Script(Script::Gunjala_Gondi)),
    ("Gurmukhi", GroupName::Script(Script::Gurmukhi)),
    ("Guru", GroupName::Script(Script::Gurmukhi)),
    ("Han", GroupName::Script(Script::Han)),
    ("Hang", GroupName::Script(Script::Hangul)),
    ("Hangul", GroupName::Script(Script::Hangul)),
    ("Hani", GroupName::Script(Script::Han)),
    ("Hanifi_Rohingya", GroupName::Script(Script::Hanifi_Rohingya)),
    ("Hano", GroupName::Script(Script::Hanunoo)),
    ("Hanunoo", GroupName::Script(Script::Hanunoo)),
    ("Hatr", GroupName::Script(Script::Hatran)),
    ("Hatran", GroupName::Script(Script::Hatran)),
    ("Hebr", GroupName::Script(Script::Hebrew)),
    ("Hebrew", GroupName::Script(Script::Hebrew)),
    ("Hex_Digit", GroupName::OtherProperties(OtherProperties::Hex_Digit)),
    ("Hira", GroupName::Script(Script::Hiragana)),
    ("Hiragana", GroupName::Script(Script::Hiragana)),
    ("Hluw", GroupName::Script(Script::Anatolian_Hieroglyphs)),
    ("Hmng", GroupName::Script(Script::Pahawh_Hmong)),
    ("Hmnp", GroupName::Script(Script::Nyiakeng_Puachue_Hmong)),
    ("Hung", GroupName::Script(Script::Old_Hungarian)),
    ("IDS_Binary_Operator", GroupName::OtherProperties(OtherProperties::IDS_Binary_Operator)),
    ("IDS_Trinary_Operator", GroupName::OtherProperties(OtherProperties::IDS_Trinary_Operator)),
    ("ID_Continue", GroupName::OtherProperties(OtherProperties::ID_Continue)),
    ("ID_Start", GroupName::OtherProperties(OtherProperties::ID_Start)),
    ("Ideographic", GroupName::OtherProperties(OtherProperties::Ideographic)),
    ("Imperial_Aramaic", GroupName::Script(Script::Imperial_Aramaic)),
    (
        "InAlphabetic_Presentation_Forms",
        GroupName::CodeBlock(CodeBlock::Alphabetic_Presentation_Forms),
    ),
    ("InArabic", GroupName::CodeBlock(CodeBlock::Arabic)),
    ("InArabic_Presentation_Forms_A", GroupName::CodeBlock(CodeBlock::Arabic_Presentation_Forms_A)),
    ("InArabic_Presentation_Forms_B", GroupName::CodeBlock(CodeBlock::Arabic_Presentation_Forms_B)),
    ("InArmenian", GroupName::CodeBlock(CodeBlock::Armenian)),
    ("InArrows", GroupName::CodeBlock(CodeBlock::Arrows)),
    ("InBasic_Latin", GroupName::CodeBlock(CodeBlock::Basic_Latin)),
    ("InBengali", GroupName::CodeBlock(CodeBlock::Bengali)),
    ("InBlock_Elements", GroupName::CodeBlock(CodeBlock::Block_Elements)),
    ("InBopomofo", GroupName::CodeBlock(CodeBlock::Bopomofo)),
    ("InBopomofo_Extended", GroupName::CodeBlock(CodeBlock::Bopomofo_Extended)),
    ("InBox_Drawing", GroupName::CodeBlock(CodeBlock::Box_Drawing)),
    ("InBraille_Patterns", GroupName::CodeBlock(CodeBlock::Braille_Patterns)),
    ("InBuhid", GroupName::CodeBlock(CodeBlock::Buhid)),
    ("InCJK_Compatibility", GroupName::CodeBlock(CodeBlock::CJK_Compatibility)),
    ("InCJK_Compatibility_Forms", GroupName::CodeBlock(CodeBlock::CJK_Compatibility_Forms)),
    (
        "InCJK_Compatibility_Ideographs",
        GroupName::CodeBlock(CodeBlock::CJK_Compatibility_Ideographs),
    ),
    ("InCJK_Radicals_Supplement", GroupName::CodeBlock(CodeBlock::CJK_Radicals_Supplement)),
    ("InCJK_Symbols_and_Punctuation", GroupName::CodeBlock(CodeBlock::CJK_Symbols_and_Punctuation)),
    ("InCJK_Unified_Ideographs", GroupName::CodeBlock(CodeBlock::CJK_Unified_Ideographs)),
    (
        "InCJK_Unified_Ideographs_Extension_A",
        GroupName::CodeBlock(CodeBlock::CJK_Unified_Ideographs_Extension_A),
    ),
    ("InCherokee", GroupName::CodeBlock(CodeBlock::Cherokee)),
    ("InCombining_Diacritical_Marks", GroupName::CodeBlock(CodeBlock::Combining_Diacritical_Marks)),
    (
        "InCombining_Diacritical_Marks_for_Symbols",
        GroupName::CodeBlock(CodeBlock::Combining_Diacritical_Marks_for_Symbols),
    ),
    ("InCombining_Half_Marks", GroupName::CodeBlock(CodeBlock::Combining_Half_Marks)),
    ("InControl_Pictures", GroupName::CodeBlock(CodeBlock::Control_Pictures)),
    ("InCurrency_Symbols", GroupName::CodeBlock(CodeBlock::Currency_Symbols)),
    ("InCyrillic", GroupName::CodeBlock(CodeBlock::Cyrillic)),
    ("InCyrillic_Supplementary", GroupName::CodeBlock(CodeBlock::Cyrillic_Supplementary)),
    ("InDevanagari", GroupName::CodeBlock(CodeBlock::Devanagari)),
    ("InDingbats", GroupName::CodeBlock(CodeBlock::Dingbats)),
    ("InEnclosed_Alphanumerics", GroupName::CodeBlock(CodeBlock::Enclosed_Alphanumerics)),
    (
        "InEnclosed_CJK_Letters_and_Months",
        GroupName::CodeBlock(CodeBlock::Enclosed_CJK_Letters_and_Months),
    ),
    ("InEthiopic", GroupName::CodeBlock(CodeBlock::Ethiopic)),
    ("InGeneral_Punctuation", GroupName::CodeBlock(CodeBlock::General_Punctuation)),
    ("InGeometric_Shapes", GroupName::CodeBlock(CodeBlock::Geometric_Shapes)),
    ("InGeorgian", GroupName::CodeBlock(CodeBlock::Georgian)),
    ("InGreek_Extended", GroupName::CodeBlock(CodeBlock::Greek_Extended)),
    ("InGreek_and_Coptic", GroupName::CodeBlock(CodeBlock::Greek_and_Coptic)),
    ("InGujarati", GroupName::CodeBlock(CodeBlock::Gujarati)),
    ("InGurmukhi", GroupName::CodeBlock(CodeBlock::Gurmukhi)),
    (
        "InHalfwidth_and_Fullwidth_Forms",
        GroupName::CodeBlock(CodeBlock::Halfwidth_and_Fullwidth_Forms),
    ),
    ("InHangul_Compatibility_Jamo", GroupName::CodeBlock(CodeBlock::Hangul_Compatibility_Jamo)),
    ("InHangul_Jamo", GroupName::CodeBlock(CodeBlock::Hangul_Jamo)),
    ("InHangul_Syllables", GroupName::CodeBlock(CodeBlock::Hangul_Syllables)),
    ("InHanunoo", GroupName::CodeBlock(CodeBlock::Hanunoo)),
    ("InHebrew", GroupName::CodeBlock(CodeBlock::Hebrew)),
    ("InHigh_Private_Use_Surrogates", GroupName::CodeBlock(CodeBlock::High_Private_Use_Surrogates)),
    ("InHigh_Surrogates", GroupName::CodeBlock(CodeBlock::High_Surrogates)),
    ("InHiragana", GroupName::CodeBlock(CodeBlock::Hiragana)),
    ("InIPA_Extensions", GroupName::CodeBlock(CodeBlock::IPA_Extensions)),
    (
        "InIdeographic_Description_Characters",
        GroupName::CodeBlock(CodeBlock::Ideographic_Description_Characters),
    ),
    ("InKanbun", GroupName::CodeBlock(CodeBlock::Kanbun)),
    ("InKangxi_Radicals", GroupName::CodeBlock(CodeBlock::Kangxi_Radicals)),
    ("InKannada", GroupName::CodeBlock(CodeBlock::Kannada)),
    ("InKatakana", GroupName::CodeBlock(CodeBlock::Katakana)),
    (
        "InKatakana_Phonetic_Extensions",
        GroupName::CodeBlock(CodeBlock::Katakana_Phonetic_Extensions),
    ),
    ("InKhmer", GroupName::CodeBlock(CodeBlock::Khmer)),
    ("InKhmer_Symbols", GroupName::CodeBlock(CodeBlock::Khmer_Symbols)),
    ("InLao", GroupName::CodeBlock(CodeBlock::Lao)),
    ("InLatin_1_Supplement", GroupName::CodeBlock(CodeBlock::Latin_1_Supplement)),
    ("InLatin_Extended_A", GroupName::CodeBlock(CodeBlock::Latin_Extended_A)),
    ("InLatin_Extended_Additional", GroupName::CodeBlock(CodeBlock::Latin_Extended_Additional)),
    ("InLatin_Extended_B", GroupName::CodeBlock(CodeBlock::Latin_Extended_B)),
    ("InLetterlike_Symbols", GroupName::CodeBlock(CodeBlock::Letterlike_Symbols)),
    ("InLimbu", GroupName::CodeBlock(CodeBlock::Limbu)),
    ("InLow_Surrogates", GroupName::CodeBlock(CodeBlock::Low_Surrogates)),
    ("InMalayalam", GroupName::CodeBlock(CodeBlock::Malayalam)),
    ("InMathematical_Operators", GroupName::CodeBlock(CodeBlock::Mathematical_Operators)),
    (
        "InMiscellaneous_Mathematical_Symbols_A",
        GroupName::CodeBlock(CodeBlock::Miscellaneous_Mathematical_Symbols_A),
    ),
    (
        "InMiscellaneous_Mathematical_Symbols_B",
        GroupName::CodeBlock(CodeBlock::Miscellaneous_Mathematical_Symbols_B),
    ),
    ("InMiscellaneous_Symbols", GroupName::CodeBlock(CodeBlock::Miscellaneous_Symbols)),
    (
        "InMiscellaneous_Symbols_and_Arrows",
        GroupName::CodeBlock(CodeBlock::Miscellaneous_Symbols_and_Arrows),
    ),
    ("InMiscellaneous_Technical", GroupName::CodeBlock(CodeBlock::Miscellaneous_Technical)),
    ("InMongolian", GroupName::CodeBlock(CodeBlock::Mongolian)),
    ("InMyanmar", GroupName::CodeBlock(CodeBlock::Myanmar)),
    ("InNumber_Forms", GroupName::CodeBlock(CodeBlock::Number_Forms)),
    ("InOgham", GroupName::CodeBlock(CodeBlock::Ogham)),
    (
        "InOptical_Character_Recognition",
        GroupName::CodeBlock(CodeBlock::Optical_Character_Recognition),
    ),
    ("InOriya", GroupName::CodeBlock(CodeBlock::Oriya)),
    ("InPhonetic_Extensions", GroupName::CodeBlock(CodeBlock::Phonetic_Extensions)),
    ("InPrivate_Use_Area", GroupName::CodeBlock(CodeBlock::Private_Use_Area)),
    ("InRunic", GroupName::CodeBlock(CodeBlock::Runic)),
    ("InSinhala", GroupName::CodeBlock(CodeBlock::Sinhala)),
    ("InSmall_Form_Variants", GroupName::CodeBlock(CodeBlock::Small_Form_Variants)),
    ("InSpacing_Modifier_Letters", GroupName::CodeBlock(CodeBlock::Spacing_Modifier_Letters)),
    ("InSpecials", GroupName::CodeBlock(CodeBlock::Specials)),
    ("InSuperscripts_and_Subscripts", GroupName::CodeBlock(CodeBlock::Superscripts_and_Subscripts)),
    ("InSupplemental_Arrows_A", GroupName::CodeBlock(CodeBlock::Supplemental_Arrows_A)),
    ("InSupplemental_Arrows_B", GroupName::CodeBlock(CodeBlock::Supplemental_Arrows_B)),
    (
        "InSupplemental_Mathematical_Operators",
        GroupName::CodeBlock(CodeBlock::Supplemental_Mathematical_Operators),
    ),
    ("InSyriac", GroupName::CodeBlock(CodeBlock::Syriac)),
    ("InTagalog", GroupName::CodeBlock(CodeBlock::Tagalog)),
    ("InTagbanwa", GroupName::CodeBlock(CodeBlock::Tagbanwa)),
    ("InTai_Le", GroupName::CodeBlock(CodeBlock::Tai_Le)),
    ("InTamil", GroupName::CodeBlock(CodeBlock::Tamil)),
    ("InTelugu", GroupName::CodeBlock(CodeBlock::Telugu)),
    ("InThaana", GroupName::CodeBlock(CodeBlock::Thaana)),
    ("InThai", GroupName::CodeBlock(CodeBlock::Thai)),
    ("InTibetan", GroupName::CodeBlock(CodeBlock::Tibetan)),
    (
        "InUnified_Canadian_Aboriginal_Syllabics",
        GroupName::CodeBlock(CodeBlock::Unified_Canadian_Aboriginal_Syllabics),
    ),
    ("InVariation_Selectors", GroupName::CodeBlock(CodeBlock::Variation_Selectors)),
    ("InYi_Radicals", GroupName::CodeBlock(CodeBlock::Yi_Radicals)),
    ("InYi_Syllables", GroupName::CodeBlock(CodeBlock::Yi_Syllables)),
    ("InYijing_Hexagram_Symbols", GroupName::CodeBlock(CodeBlock::Yijing_Hexagram_Symbols)),
    ("Inherited", GroupName::Script(Script::Inherited)),
    ("Initial_Punctuation", GroupName::Category(Category::Initial_Punctuation)),
    ("Inscriptional_Pahlavi", GroupName::Script(Script::Inscriptional_Pahlavi)),
    ("Inscriptional_Parthian", GroupName::Script(Script::Inscriptional_Parthian)),
    ("Ital", GroupName::Script(Script::Old_Italic)),
    ("Java", GroupName::Script(Script::Javanese)),
    ("Javanese", GroupName::Script(Script::Javanese)),
    ("Join_Control", GroupName::OtherProperties(OtherProperties::Join_Control)),
    ("Joining_Group", GroupName::OtherProperties(OtherProperties::Joining_Group)),
    ("Kaithi", GroupName::Script(Script::Kaithi)),
    ("Kali", GroupName::Script(Script::Kayah_Li)),
    ("Kana", GroupName::Script(Script::Katakana)),
    ("Kannada", GroupName::Script(Script::Kannada)),
    ("Katakana", GroupName::Script(Script::Katakana)),
    ("Kayah_Li", GroupName::Script(Script::Kayah_Li)),
    ("Khar", GroupName::Script(Script::Kharoshthi)),
    ("Kharoshthi", GroupName::Script(Script::Kharoshthi)),
    ("Khitan_Small_Script", GroupName::Script(Script::Khitan_Small_Script)),
    ("Khmer", GroupName::Script(Script::Khmer)),
    ("Khmr", GroupName::Script(Script::Khmer)),
    ("Khoj", GroupName::Script(Script::Khojki)),
    ("Khojki", GroupName::Script(Script::Khojki)),
    ("Khudawadi", GroupName::Script(Script::Khudawadi)),
    ("Kits", GroupName::Script(Script::Khitan_Small_Script)),
    ("Knda", GroupName::Script(Script::Kannada)),
    ("Kthi", GroupName::Script(Script::Kaithi)),
    ("L", GroupName::Category(Category::Letter)),
    ("LC", GroupName::Category(Category::Cased_Letter)),
    ("Lana", GroupName::Script(Script::Tai_Tham)),
    ("Lao", GroupName::Script(Script::Lao)),
    ("Laoo", GroupName::Script(Script::Lao)),
    ("Latin", GroupName::Script(Script::Latin)),
    ("Latn", GroupName::Script(Script::Latin)),
    ("Lepc", GroupName::Script(Script::Lepcha)),
    ("Lepcha", GroupName::Script(Script::Lepcha)),
    ("Letter", GroupName::Category(Category::Letter)),
    ("Letter_Number", GroupName::Category(Category::Letter_Number)),
    ("Limb", GroupName::Script(Script::Limbu)),
    ("Limbu", GroupName::Script(Script::Limbu)),
    ("Lina", GroupName::Script(Script::Linear_A)),
    ("Linb", GroupName::Script(Script::Linear_B)),
    ("Line_Separator", GroupName::Category(Category::Line_Separator)),
    ("Linear_A", GroupName::Script(Script::Linear_A)),
    ("Linear_B", GroupName::Script(Script::Linear_B)),
    ("Lisu", GroupName::Script(Script::Lisu)),
    ("Ll", GroupName::Category(Category::Lowercase_Letter)),
    ("Lm", GroupName::Category(Category::Modifier_Letter)),
    ("Lo", GroupName::Category(Category::Other_Letter)),
    (
        "Logical_Order_Exception",
        GroupName::OtherProperties(OtherProperties::Logical_Order_Exception),
    ),
    ("Lowercase", GroupName::OtherProperties(OtherProperties::Lowercase)),
    ("Lowercase_Letter", GroupName::Category(Category::Lowercase_Letter)),
    ("Lt", GroupName::Category(Category::Titlecase_Letter)),
    ("Lu", GroupName::Category(Category::Uppercase_Letter)),
    ("Lyci", GroupName::Script(Script::Lycian)),
    ("Lycian", GroupName::Script(Script::Lycian)),
    ("Lydi", GroupName::Script(Script::Lydian)),
    ("Lydian", GroupName::Script(Script::Lydian)),
    ("M", GroupName::Category(Category::Mark)),
    ("Mahajani", GroupName::Script(Script::Mahajani)),
    ("Mahj", GroupName::Script(Script::Mahajani)),
    ("Maka", GroupName::Script(Script::Makasar)),
    ("Makasar", GroupName::Script(Script::Makasar)),
    ("Malayalam", GroupName::Script(Script::Malayalam)),
    ("Mand", GroupName::Script(Script::Mandaic)),
    ("Mandaic", GroupName::Script(Script::Mandaic)),
    ("Mani", GroupName::Script(Script::Manichaean)),
    ("Manichaean", GroupName::Script(Script::Manichaean)),
    ("Marc", GroupName::Script(Script::Marchen)),
    ("Marchen", GroupName::Script(Script::Marchen)),
    ("Mark", GroupName::Category(Category::Mark)),
    ("Masaram_Gondi", GroupName::Script(Script::Masaram_Gondi)),
    ("Math", GroupName::OtherProperties(OtherProperties::Math)),
    ("Math_Symbol", GroupName::Category(Category::Math_Symbol)),
    ("Mc", GroupName::Category(Category::Spacing_Mark)),
    ("Me", GroupName::Category(Category::Enclosing_Mark)),
    ("Medefaidrin", GroupName::Script(Script::Medefaidrin)),
    ("Medf", GroupName::Script(Script::Medefaidrin)),
    ("Meetei_Mayek", GroupName::Script(Script::Meetei_Mayek)),
    ("Mend", GroupName::Script(Script::Mende_Kikakui)),
    ("Mende_Kikakui", GroupName::Script(Script::Mende_Kikakui)),
    ("Merc", GroupName::Script(Script::Meroitic_Cursive)),
    ("Mero", GroupName::Script(Script::Meroitic_Hieroglyphs)),
    ("Meroitic_Cursive", GroupName::Script(Script::Meroitic_Cursive)),
    ("Meroitic_Hieroglyphs", GroupName::Script(Script::Meroitic_Hieroglyphs)),
    ("Miao", GroupName::Script(Script::Miao)),
    ("Mlym", GroupName::Script(Script::Malayalam)),
    ("Mn", GroupName::Category(Category::Nonspacing_Mark)),
    ("Modi", GroupName::Script(Script::Modi)),
    ("Modifier_Letter", GroupName::Category(Category::Modifier_Letter)),
    ("Modifier_Symbol", GroupName::Category(Category::Modifier_Symbol)),
    ("Mong", GroupName::Script(Script::Mongolian)),
    ("Mongolian", GroupName::Script(Script::Mongolian)),
    ("Mro", GroupName::Script(Script::Mro)),
    ("Mroo", GroupName::Script(Script::Mro)),
    ("Mtei", GroupName::Script(Script::Meetei_Mayek)),
    ("Mult", GroupName::Script(Script::Multani)),
    ("Multani", GroupName::Script(Script::Multani)),
    ("Myanmar", GroupName::Script(Script::Myanmar)),
    ("Mymr", GroupName::Script(Script::Myanmar)),
    ("N", GroupName::Category(Category::Number)),
    ("Nabataean", GroupName::Script(Script::Nabataean)),
    ("Nand", GroupName::Script(Script::Nandinagari)),
    ("Nandinagari", GroupName::Script(Script::Nandinagari)),
    ("Narb", GroupName::Script(Script::Old_North_Arabian)),
    ("Nbat", GroupName::Script(Script::Nabataean)),
    ("Nd", GroupName::Category(Category::Decimal_Number)),
    ("New_Tai_Lue", GroupName::Script(Script::New_Tai_Lue)),
    ("Newa", GroupName::Script(Script::Newa)),
    ("Nko", GroupName::Script(Script::Nko)),
    ("Nkoo", GroupName::Script(Script::Nko)),
    ("Nl", GroupName::Category(Category::Letter_Number)),
    ("No", GroupName::Category(Category::Other_Number)),
    (
        "Noncharacter_Code_Point",
        GroupName::OtherProperties(OtherProperties::Noncharacter_Code_Point),
    ),
    ("Nonspacing_Mark", GroupName::Category(Category::Nonspacing_Mark)),
    ("Nshu", GroupName::Script(Script::Nushu)),
    ("Number", GroupName::Category(Category::Number)),
    ("Nushu", GroupName::Script(Script::Nushu)),
    ("Nyiakeng_Puachue_Hmong", GroupName::Script(Script::Nyiakeng_Puachue_Hmong)),
    ("Ogam", GroupName::Script(Script::Ogham)),
    ("Ogham", GroupName::Script(Script::Ogham)),
    ("Ol_Chiki", GroupName::Script(Script::Ol_Chiki)),
    ("Olck", GroupName::Script(Script::Ol_Chiki)),
    ("Old_Hungarian", GroupName::Script(Script::Old_Hungarian)),
    ("Old_Italic", GroupName::Script(Script::Old_Italic)),
    ("Old_North_Arabian", GroupName::Script(Script::Old_North_Arabian)),
    ("Old_Permic", GroupName::Script(Script::Old_Permic)),
    ("Old_Persian", GroupName::Script(Script::Old_Persian)),
    ("Old_Sogdian", GroupName::Script(Script::Old_Sogdian)),
    ("Old_South_Arabian", GroupName::Script(Script::Old_South_Arabian)),
    ("Old_Turkic", GroupName::Script(Script::Old_Turkic)),
    ("Old_Uyghur", GroupName::Script(Script::Old_Uyghur)),
    ("Open_Punctuation", GroupName::Category(Category::Open_Punctuation)),
    ("Oriya", GroupName::Script(Script::Oriya)),
    ("Orkh", GroupName::Script(Script::Old_Turkic)),
    ("Orya", GroupName::Script(Script::Oriya)),
    ("Osage", GroupName::Script(Script::Osage)),
    ("Osge", GroupName::Script(Script::Osage)),
    ("Osma", GroupName::Script(Script::Osmanya)),
    ("Osmanya", GroupName::Script(Script::Osmanya)),
    ("Other", GroupName::Category(Category::Other)),
    ("Other_Letter", GroupName::Category(Category::Other_Letter)),
    ("Other_Number", GroupName::Category(Category::Other_Number)),
    ("Other_Punctuation", GroupName::Category(Category::Other_Punctuation)),
    ("Other_Symbol", GroupName::Category(Category::Other_Symbol)),
    ("Ougr", GroupName::Script(Script::Old_Uyghur)),
    ("P", GroupName::Category(Category::Punctuation)),
    ("Pahawh_Hmong", GroupName::Script(Script::Pahawh_Hmong)),
    ("Palm", GroupName::Script(Script::Palmyrene)),
    ("Palmyrene", GroupName::Script(Script::Palmyrene)),
    ("Paragraph_Separator", GroupName::Category(Category::Paragraph_Separator)),
    ("Pattern_Syntax", GroupName::OtherProperties(OtherProperties::Pattern_Syntax)),
    ("Pattern_White_Space", GroupName::OtherProperties(OtherProperties::Pattern_White_Space)),
    ("Pau_Cin_Hau", GroupName::Script(Script::Pau_Cin_Hau)),
    ("Pauc", GroupName::Script(Script::Pau_Cin_Hau)),
    ("Pc", GroupName::Category(Category::Connector_Punctuation)),
    ("Pd", GroupName::Category(Category::Dash_Punctuation)),
    ("Pe", GroupName::Category(Category::Close_Punctuation)),
    ("Perm", GroupName::Script(Script::Old_Permic)),
    ("Pf", GroupName::Category(Category::Final_Punctuation)),
    ("Phag", GroupName::Script(Script::Phags_Pa)),
    ("Phags_Pa", GroupName::Script(Script::Phags_Pa)),
    ("Phli", GroupName::Script(Script::Inscriptional_Pahlavi)),
    ("Phlp", GroupName::Script(Script::Psalter_Pahlavi)),
    ("Phnx", GroupName::Script(Script::Phoenician)),
    ("Phoenician", GroupName::Script(Script::Phoenician)),
    ("Pi", GroupName::Category(Category::Initial_Punctuation)),
    ("Plrd", GroupName::Script(Script::Miao)),
    ("Po", GroupName::Category(Category::Other_Punctuation)),
    ("Private_Use", GroupName::Category(Category::Private_Use)),
    ("Prti", GroupName::Script(Script::Inscriptional_Parthian)),
    ("Ps", GroupName::Category(Category::Open_Punctuation)),
    ("Psalter_Pahlavi", GroupName::Script(Script::Psalter_Pahlavi)),
    ("Punctuation", GroupName::Category(Category::Punctuation)),
    ("Qaac", GroupName::Script(Script::Coptic)),
    ("Qaai", GroupName::Script(Script::Inherited)),
    ("Quotation_Mark", GroupName::OtherProperties(OtherProperties::Quotation_Mark)),
    ("Radical", GroupName::OtherProperties(OtherProperties::Radical)),
    ("Regional_Indicator", GroupName::OtherProperties(OtherProperties::Regional_Indicator)),
    ("Rejang", GroupName::Script(Script::Rejang)),
    ("Rjng", GroupName::Script(Script::Rejang)),
    ("Rohg", GroupName::Script(Script::Hanifi_Rohingya)),
    ("Runic", GroupName::Script(Script::Runic)),
    ("Runr", GroupName::Script(Script::Runic)),
    ("S", GroupName::Category(Category::Symbol)),
    ("Samaritan", GroupName::Script(Script::Samaritan)),
    ("Samr", GroupName::Script(Script::Samaritan)),
    ("Sarb", GroupName::Script(Script::Old_South_Arabian)),
    ("Saur", GroupName::Script(Script::Saurashtra)),
    ("Saurashtra", GroupName::Script(Script::Saurashtra)),
    ("Sc", GroupName::Category(Category::Currency_Symbol)),
    ("Sentence_Terminal", GroupName::OtherProperties(OtherProperties::Sentence_Terminal)),
    ("Separator", GroupName::Category(Category::Separator)),
    ("Sgnw", GroupName::Script(Script::SignWriting)),
    ("Sharada", GroupName::Script(Script::Sharada)),
    ("Shavian", GroupName::Script(Script::Shavian)),
    ("Shaw", GroupName::Script(Script::Shavian)),
    ("Shrd", GroupName::Script(Script::Sharada)),
    ("Sidd", GroupName::Script(Script::Siddham)),
    ("Siddham", GroupName::Script(Script::Siddham)),
    ("SignWriting", GroupName::Script(Script::SignWriting)),
    ("Sind", GroupName::Script(Script::Khudawadi)),
    ("Sinh", GroupName::Script(Script::Sinhala)),
    ("Sinhala", GroupName::Script(Script::Sinhala)),
    ("Sk", GroupName::Category(Category::Modifier_Symbol)),
    ("Sm", GroupName::Category(Category::Math_Symbol)),
    ("So", GroupName::Category(Category::Other_Symbol)),
    ("Soft_Dotted", GroupName::OtherProperties(OtherProperties::Soft_Dotted)),
    ("Sogd", GroupName::Script(Script::Sogdian)),
    ("Sogdian", GroupName::Script(Script::Sogdian)),
    ("Sogo", GroupName::Script(Script::Old_Sogdian)),
    ("Sora", GroupName::Script(Script::Sora_Sompeng)),
    ("Sora_Sompeng", GroupName::Script(Script::Sora_Sompeng)),
    ("Soyo", GroupName::Script(Script::Soyombo)),
    ("Soyombo", GroupName::Script(Script::Soyombo)),
    ("Space_Separator", GroupName::Category(Category::Space_Separator)),
    ("Spacing_Mark", GroupName::Category(Category::Spacing_Mark)),
    ("Sund", GroupName::Script(Script::Sundanese)),
    ("Sundanese", GroupName::Script(Script::Sundanese)),
    ("Surrogate", GroupName::Category(Category::Surrogate)),
    ("Sylo", GroupName::Script(Script::Syloti_Nagri)),
    ("Syloti_Nagri", GroupName::Script(Script::Syloti_Nagri)),
    ("Symbol", GroupName::Category(Category::Symbol)),
    ("Syrc", GroupName::Script(Script::Syriac)),
    ("Syriac", GroupName::Script(Script::Syriac)),
    ("Tagalog", GroupName::Script(Script::Tagalog)),
    ("Tagb", GroupName::Script(Script::Tagbanwa)),
    ("Tagbanwa", GroupName::Script(Script::Tagbanwa)),
    ("Tai_Le", GroupName::Script(Script::Tai_Le)),
    ("Tai_Tham", GroupName::Script(Script::Tai_Tham)),
    ("Tai_Viet", GroupName::Script(Script::Tai_Viet)),
    ("Takr", GroupName::Script(Script::Takri)),
    ("Takri", GroupName::Script(Script::Takri)),
    ("Tale", GroupName::Script(Script::Tai_Le)),
    ("Talu", GroupName::Script(Script::New_Tai_Lue)),
    ("Tamil", GroupName::Script(Script::Tamil)),
    ("Taml", GroupName::Script(Script::Tamil)),
    ("Tang", GroupName::Script(Script::Tangut)),
    ("Tangsa", GroupName::Script(Script::Tangsa)),
    ("Tangut", GroupName::Script(Script::Tangut)),
    ("Tavt", GroupName::Script(Script::Tai_Viet)),
    ("Telu", GroupName::Script(Script::Telugu)),
    ("Telugu", GroupName::Script(Script::Telugu)),
    ("Terminal_Punctuation", GroupName::OtherProperties(OtherProperties::Terminal_Punctuation)),
    ("Tfng", GroupName::Script(Script::Tifinagh)),
    ("Tglg", GroupName::Script(Script::Tagalog)),
    ("Thaa", GroupName::Script(Script::Thaana)),
    ("Thaana", GroupName::Script(Script::Thaana)),
    ("Thai", GroupName::Script(Script::Thai)),
    ("Tibetan", GroupName::Script(Script::Tibetan)),
    ("Tibt", GroupName::Script(Script::Tibetan)),
    ("Tifinagh", GroupName::Script(Script::Tifinagh)),
    ("Tirh", GroupName::Script(Script::Tirhuta)),
    ("Tirhuta", GroupName::Script(Script::Tirhuta)),
    ("Titlecase_Letter", GroupName::Category(Category::Titlecase_Letter)),
    ("Tnsa", GroupName::Script(Script::Tangsa)),
    ("Toto", GroupName::Script(Script::Toto)),
    ("Ugar", GroupName::Script(Script::Ugaritic)),
    ("Ugaritic", GroupName::Script(Script::Ugaritic)),
    ("Unassigned", GroupName::Category(Category::Unassigned)),
    ("Unified_Ideograph", GroupName::OtherProperties(OtherProperties::Unified_Ideograph)),
    ("Uppercase", GroupName::OtherProperties(OtherProperties::Uppercase)),
    ("Uppercase_Letter", GroupName::Category(Category::Uppercase_Letter)),
    ("Vai", GroupName::Script(Script::Vai)),
    ("Vaii", GroupName::Script(Script::Vai)),
    ("Variation_Selector", GroupName::OtherProperties(OtherProperties::Variation_Selector)),
    ("Vith", GroupName::Script(Script::Vithkuqi)),
    ("Vithkuqi", GroupName::Script(Script::Vithkuqi)),
    ("Wancho", GroupName::Script(Script::Wancho)),
    ("Wara", GroupName::Script(Script::Warang_Citi)),
    ("Warang_Citi", GroupName::Script(Script::Warang_Citi)),
    ("Wcho", GroupName::Script(Script::Wancho)),
    ("White_Space", GroupName::OtherProperties(OtherProperties::White_Space)),
    ("XID_Continue", GroupName::OtherProperties(OtherProperties::XID_Continue)),
    ("XID_Start", GroupName::OtherProperties(OtherProperties::XID_Start)),
    ("Xpeo", GroupName::Script(Script::Old_Persian)),
    ("Xsux", GroupName::Script(Script::Cuneiform)),
    ("Yezi", GroupName::Script(Script::Yezidi)),
    ("Yezidi", GroupName::Script(Script::Yezidi)),
    ("Yi", GroupName::Script(Script::Yi)),
    ("Yiii", GroupName::Script(Script::Yi)),
    ("Z", GroupName::Category(Category::Separator)),
    ("Zanabazar_Square", GroupName::Script(Script::Zanabazar_Square)),
    ("Zanb", GroupName::Script(Script::Zanabazar_Square)),
    ("Zinh", GroupName::Script(Script::Inherited)),
    ("Zl", GroupName::Category(Category::Line_Separator)),
    ("Zp", GroupName::Category(Category::Paragraph_Separator)),
    ("Zs", GroupName::Category(Category::Space_Separator)),
    ("Zyyy", GroupName::Script(Script::Common)),
    ("cntrl", GroupName::Category(Category::Control)),
    ("d", GroupName::Category(Category::Decimal_Number)),
    ("digit", GroupName::Category(Category::Decimal_Number)),
    ("h", (GroupName::HorizSpace)),
    ("horiz_space", (GroupName::HorizSpace)),
    ("l", (GroupName::LineBreak)),
    ("line_break", (GroupName::LineBreak)),
    ("punct", GroupName::Category(Category::Punctuation)),
    ("s", GroupName::Category(Category::Separator)),
    ("space", GroupName::Category(Category::Separator)),
    ("v", (GroupName::VertSpace)),
    ("vert_space", (GroupName::VertSpace)),
    ("w", (GroupName::Word)),
    ("word", (GroupName::Word)),
];
