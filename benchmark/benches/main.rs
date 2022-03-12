use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rulex::Rulex;

const PARSE_INPUT: &str = r#"
[.] [w] [s] [cp] [ascii_alpha] [.] [w] [s] [cp] [Latn] [punct]
(['ab'] | ['+-*/%' '[' ']' '0'-'9'] | ['f'-'o' 'j'-'z'])
([.] | [w] | :() | % "tests" % | % "test" %)
((((((((((((((((((((((((('a')))))))))))))))))))))))))
:(:(:(:(:(:(:(:(:(:(:(:(:(:(:(:(:(:(:(:(:(:(:(:(:("test")))))))))))))))))))))))))
"hello"{2}{3}{4}{5}{6}?{1,4}{2,9} greedy {,3}* greedy
"#;

const NAMES_INPUT: &str = "
[Adlam] [Adlm] [Aghb] [Ahom] [Anatolian_Hieroglyphs] [Arab] [Arabic] [Armenian] [Armi] [Armn]
[Avestan] [Avst] [Bali] [Balinese] [Bamu] [Bamum] [Bass] [Bassa_Vah] [Batak] [Batk] [Beng] [Bengali]
[Bhaiksuki] [Bhks] [Bopo] [Bopomofo] [Brah] [Brahmi] [Brai] [Braille] [Bugi] [Buginese] [Buhd]
[Buhid] [C] [Cakm] [Canadian_Aboriginal] [Cans] [Cari] [Carian] [Cased_Letter] [Caucasian_Albanian]
[Cc] [Cf] [Chakma] [Cham] [Cher] [Cherokee] [Chorasmian] [Chrs] [Close_Punctuation] [Cn] [Co]
[Combining_Mark] [Common] [Connector_Punctuation] [Control] [Copt] [Coptic] [Cpmn] [Cprt] [Cs]
[Cuneiform] [Currency_Symbol] [Cypriot] [Cypro_Minoan] [Cyrillic] [Cyrl] [Dash_Punctuation]
[Decimal_Number] [Deseret] [Deva] [Devanagari] [Diak] [Dives_Akuru] [Dogr] [Dogra] [Dsrt] [Dupl]
[Duployan] [Egyp] [Egyptian_Hieroglyphs] [Elba] [Elbasan] [Elym] [Elymaic] [Enclosing_Mark] [Ethi]
[Ethiopic] [Final_Punctuation] [Format] [Geor] [Georgian] [Glag] [Glagolitic] [Gong] [Gonm] [Goth]
[Gothic] [Gran] [Grantha] [Greek] [Grek] [Gujarati] [Gujr] [Gunjala_Gondi] [Gurmukhi] [Guru] [Han]
[Hang] [Hangul] [Hani] [Hanifi_Rohingya] [Hano] [Hanunoo] [Hatr] [Hatran] [Hebr] [Hebrew] [Hira]
[Hiragana] [Hluw] [Hmng] [Hmnp] [Hung] [Imperial_Aramaic] [InAlphabetic_Presentation_Forms]
[InArabic] [InArabic_Presentation_Forms_A] [InArabic_Presentation_Forms_B] [InArmenian] [InArrows]
[InBasic_Latin] [InBengali] [InBlock_Elements] [InBopomofo] [InBopomofo_Extended] [InBox_Drawing]
[InBraille_Patterns] [InBuhid] [InCJK_Compatibility] [InCJK_Compatibility_Forms]
[InCJK_Compatibility_Ideographs] [InCJK_Radicals_Supplement] [InCJK_Symbols_and_Punctuation]
[InCJK_Unified_Ideographs] [InCJK_Unified_Ideographs_Extension_A] [InCherokee]
[InCombining_Diacritical_Marks] [InCombining_Diacritical_Marks_for_Symbols] [InCombining_Half_Marks]
[InControl_Pictures] [InCurrency_Symbols] [InCyrillic] [InCyrillic_Supplementary] [InDevanagari]
[InDingbats] [InEnclosed_Alphanumerics] [InEnclosed_CJK_Letters_and_Months] [InEthiopic]
[InGeneral_Punctuation] [InGeometric_Shapes] [InGeorgian] [InGreek_Extended] [InGreek_and_Coptic]
[InGujarati] [InGurmukhi] [InHalfwidth_and_Fullwidth_Forms] [InHangul_Compatibility_Jamo]
[InHangul_Jamo] [InHangul_Syllables] [InHanunoo] [InHebrew] [InHigh_Private_Use_Surrogates]
[InHigh_Surrogates] [InHiragana] [InIPA_Extensions] [InIdeographic_Description_Characters]
[InKanbun] [InKangxi_Radicals] [InKannada] [InKatakana] [InKatakana_Phonetic_Extensions] [InKhmer]
[InKhmer_Symbols] [InLao] [InLatin_1_Supplement] [InLatin_Extended_A] [InLatin_Extended_Additional]
[InLatin_Extended_B] [InLetterlike_Symbols] [InLimbu] [InLow_Surrogates] [InMalayalam]
[InMathematical_Operators] [InMiscellaneous_Mathematical_Symbols_A]
[InMiscellaneous_Mathematical_Symbols_B] [InMiscellaneous_Symbols]
[InMiscellaneous_Symbols_and_Arrows] [InMiscellaneous_Technical] [InMongolian] [InMyanmar]
[InNumber_Forms] [InOgham] [InOptical_Character_Recognition] [InOriya] [InPhonetic_Extensions]
[InPrivate_Use_Area] [InRunic] [InSinhala] [InSmall_Form_Variants] [InSpacing_Modifier_Letters]
[InSpecials] [InSuperscripts_and_Subscripts] [InSupplemental_Arrows_A] [InSupplemental_Arrows_B]
[InSupplemental_Mathematical_Operators] [InSyriac] [InTagalog] [InTagbanwa] [InTai_Le] [InTamil]
[InTelugu] [InThaana] [InThai] [InTibetan] [InUnified_Canadian_Aboriginal_Syllabics]
[InVariation_Selectors] [InYi_Radicals] [InYi_Syllables] [InYijing_Hexagram_Symbols] [Inherited]
[Initial_Punctuation] [Inscriptional_Pahlavi] [Inscriptional_Parthian] [Ital] [Java] [Javanese]
[Kaithi] [Kali] [Kana] [Kannada] [Katakana] [Kayah_Li] [Khar] [Kharoshthi] [Khitan_Small_Script]
[Khmer] [Khmr] [Khoj] [Khojki] [Khudawadi] [Kits] [Knda] [Kthi] [L] [LC] [Lana] [Lao] [Laoo] [Latin]
[Latn] [Lepc] [Lepcha] [Letter] [Letter_Number] [Limb] [Limbu] [Lina] [Linb] [Line_Separator]
[Linear_A] [Linear_B] [Lisu] [Ll] [Lm] [Lo] [Lowercase_Letter] [Lt] [Lu] [Lyci] [Lycian] [Lydi]
[Lydian] [M] [Mahajani] [Mahj] [Maka] [Makasar] [Malayalam] [Mand] [Mandaic] [Mani] [Manichaean]
[Marc] [Marchen] [Mark] [Masaram_Gondi] [Math_Symbol] [Mc] [Me] [Medefaidrin] [Medf] [Meetei_Mayek]
[Mend] [Mende_Kikakui] [Merc] [Mero] [Meroitic_Cursive] [Meroitic_Hieroglyphs] [Miao] [Mlym] [Mn]
[Modi] [Modifier_Letter] [Modifier_Symbol] [Mong] [Mongolian] [Mro] [Mroo] [Mtei] [Mult] [Multani]
[Myanmar] [Mymr] [N] [Nabataean] [Nand] [Nandinagari] [Narb] [Nbat] [Nd] [New_Tai_Lue] [Newa] [Nko]
[Nkoo] [Nl] [No] [Nonspacing_Mark] [Nshu] [Number] [Nushu] [Nyiakeng_Puachue_Hmong] [Ogam] [Ogham]
[Ol_Chiki] [Olck] [Old_Hungarian] [Old_Italic] [Old_North_Arabian] [Old_Permic] [Old_Persian]
[Old_Sogdian] [Old_South_Arabian] [Old_Turkic] [Old_Uyghur] [Open_Punctuation] [Oriya] [Orkh]
[Orya] [Osage] [Osge] [Osma] [Osmanya] [Other] [Other_Letter] [Other_Number] [Other_Punctuation]
[Other_Symbol] [Ougr] [P] [Pahawh_Hmong] [Palm] [Palmyrene] [Paragraph_Separator] [Pau_Cin_Hau]
[Pauc] [Pc] [Pd] [Pe] [Perm] [Pf] [Phag] [Phags_Pa] [Phli] [Phlp] [Phnx] [Phoenician] [Pi] [Plrd]
[Po] [Private_Use] [Prti] [Ps] [Psalter_Pahlavi] [Punctuation] [Qaac] [Qaai] [Rejang] [Rjng] [Rohg]
[Runic] [Runr] [S] [Samaritan] [Samr] [Sarb] [Saur] [Saurashtra] [Sc] [Separator] [Sgnw] [Sharada]
[Shavian] [Shaw] [Shrd] [Sidd] [Siddham] [SignWriting] [Sind] [Sinh] [Sinhala] [Sk] [Sm] [So] [Sogd]
[Sogdian] [Sogo] [Sora] [Sora_Sompeng] [Soyo] [Soyombo] [Space_Separator] [Spacing_Mark] [Sund]
[Sundanese] [Surrogate] [Sylo] [Syloti_Nagri] [Symbol] [Syrc] [Syriac] [Tagalog] [Tagb] [Tagbanwa]
[Tai_Le] [Tai_Tham] [Tai_Viet] [Takr] [Takri] [Tale] [Talu] [Tamil] [Taml] [Tang] [Tangsa] [Tangut]
[Tavt] [Telu] [Telugu] [Tfng] [Tglg] [Thaa] [Thaana] [Thai] [Tibetan] [Tibt] [Tifinagh] [Tirh]
[Tirhuta] [Titlecase_Letter] [Tnsa] [Toto] [Ugar] [Ugaritic] [Unassigned] [Uppercase_Letter] [Vai]
[Vaii] [Vith] [Vithkuqi] [Wancho] [Wara] [Warang_Citi] [Wcho] [Xpeo] [Xsux] [Yezi] [Yezidi] [Yi]
[Yiii] [Z] [Zanabazar_Square] [Zanb] [Zinh] [Zl] [Zp] [Zs] [Zyyy] [cntrl] [digit] [punct]";

pub fn everything(c: &mut Criterion) {
    c.bench_function("parse everything", |b| {
        b.iter(|| Rulex::parse(black_box(PARSE_INPUT), Default::default()).unwrap())
    });
}

pub fn named_classes(c: &mut Criterion) {
    c.bench_function("parse named_classes", |b| {
        b.iter(|| Rulex::parse(black_box(NAMES_INPUT), Default::default()).unwrap())
    });
}

criterion_group!(benches, everything, named_classes);
criterion_main!(benches);
