use std::collections::{HashMap, HashSet};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=PropertyValueAliases.txt");
    println!("cargo:rerun-if-changed=SupportedBooleanProps.txt");
    println!("cargo:rerun-if-changed=DotNetSupportedBlocks.txt");
    generate_unicode_data();
}

fn generate_unicode_data() {
    let blocks = std::fs::read_to_string("Blocks.txt").unwrap();
    let blocks = parse_blocks(&blocks);

    let property_value_aliases = std::fs::read_to_string("PropertyValueAliases.txt").unwrap();
    let supported_boolean_props = std::fs::read_to_string("SupportedBooleanProps.txt").unwrap();
    let aliases = property_value_aliases + "\n" + &supported_boolean_props;
    let [categories, scripts, blocks, bools] = parse_aliases(&aliases, &blocks);

    let dotnet_blocks = std::fs::read_to_string("DotNetSupportedBlocks.txt").unwrap();

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let path = std::path::Path::new(&out_dir).join("unicode_data.rs");

    let mut lut = vec![
        "(\"w\", GroupName::Word)".to_string(),
        "(\"d\", GroupName::Digit)".to_string(),
        "(\"s\", GroupName::Space)".to_string(),
        "(\"h\", GroupName::HorizSpace)".to_string(),
        "(\"v\", GroupName::VertSpace)".to_string(),
        "(\"word\", GroupName::Word)".to_string(),
        "(\"digit\", GroupName::Digit)".to_string(),
        "(\"space\", GroupName::Space)".to_string(),
        "(\"horiz_space\", GroupName::HorizSpace)".to_string(),
        "(\"vert_space\", GroupName::VertSpace)".to_string(),
    ];

    // tuples: (name, is_block)
    let mut distinct_cache =
        ["w", "d", "s", "h", "v", "word", "digit", "space", "horiz_space", "vert_space"]
            .into_iter()
            .map(|name| (name, false))
            .collect::<HashSet<_>>();

    for category in &categories {
        let canonical = category[1];
        for &name in category {
            if !distinct_cache.contains(&(name, false)) {
                lut.push(format!("(\"{name}\", GroupName::Category(Category::{canonical}))"));
                distinct_cache.insert((name, false));
            }
        }
    }
    for script in &scripts {
        let canonical = script[1];
        for &name in script {
            if !distinct_cache.contains(&(name, false)) {
                lut.push(format!("(\"{name}\", GroupName::Script(Script::{canonical}))"));
                distinct_cache.insert((name, false));
            }
        }
    }
    for block in &blocks {
        let canonical = block[1].replace('-', "_");
        for &name in block {
            if !distinct_cache.contains(&(name, true)) {
                lut.push(format!(
                    "(\"In{name}\", GroupName::CodeBlock(CodeBlock::{canonical}))",
                    name = name.replace('-', "_")
                ));
                distinct_cache.insert((name, true));
            }
        }
    }
    for bool in &bools {
        let canonical = bool[1];
        for &name in bool {
            if !distinct_cache.contains(&(name, false)) {
                lut.push(format!(
                    "(\"{name}\", GroupName::OtherProperties(OtherProperties::{canonical}))"
                ));
                distinct_cache.insert((name, false));
            }
        }
    }

    lut.sort_unstable();

    std::fs::write(
        path,
        format!(
            "{category_enum}

{script_enum}

{block_enum}

{other_enum}

static PARSE_LUT: &[(&str, GroupName)] = &[
    {lut}
];

static DOTNET_SUPPORTED: &[&str] = &[
{dotnet_supported}];
",
            category_enum = generate_enum("Category", &categories, 0, 1),
            script_enum = generate_enum("Script", &scripts, 1, 1),
            block_enum = generate_enum("CodeBlock", &blocks, 1, 1),
            other_enum = generate_enum("OtherProperties", &bools, 1, 1),
            lut = lut.join(",\n    "),
            dotnet_supported =
                dotnet_blocks.lines().map(|line| format!("    {line:?},\n")).collect::<String>()
        ),
    )
    .unwrap();
}

fn generate_enum(
    name: &str,
    variants: &[Vec<impl AsRef<str>>],
    compile_index: usize,
    canonical_index: usize,
) -> String {
    format!(
        r#"#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
#[allow(clippy::enum_variant_names)]
#[repr({size})]
pub enum {name} {{ {variants} }}

impl {name} {{
    pub fn as_str(self) -> &'static str {{
        static LUT: &[&str] = &[{compiled}];
        LUT[self as {size} as usize]
    }}
}}"#,
        size = if variants.len() > 256 { "u16" } else { "u8" },
        variants = variants
            .iter()
            .map(|c| format!("{}, ", c[canonical_index].as_ref().replace('-', "_")))
            .collect::<String>(),
        compiled = variants
            .iter()
            .map(|c| format!("{:?}, ", c[compile_index].as_ref()))
            .collect::<String>(),
    )
}

fn parse_blocks(data: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();

    for mut line in data.lines() {
        if let Some(hashtag) = line.find('#') {
            line = &line[..hashtag];
        }
        line = line.trim();

        if line.is_empty() {
            continue;
        }

        let block_name = line.split(';').nth(1).unwrap().trim();
        let no_spaces = block_name.replace(' ', "_");
        let no_dashes = no_spaces.replace('-', "_");
        map.insert(no_dashes, no_spaces);
    }

    map
}

fn parse_aliases<'a>(
    data: &'a str,
    block_map: &'a HashMap<String, String>,
) -> [Vec<Vec<&'a str>>; 4] {
    let mut categories = vec![];
    let mut scripts = vec![];
    let mut blocks = vec![];
    let mut bools = vec![];

    dbg!(&block_map);

    for mut line in data.lines() {
        if let Some(hashtag) = line.find('#') {
            line = &line[..hashtag];
        }
        line = line.trim();

        if line.is_empty() {
            continue;
        }

        let mut parts = line.split(';');
        let Some(property) = parts.next() else {
            continue;
        };
        match property.trim() {
            "gc" => categories.push(parts.map(str::trim).collect::<Vec<_>>()),
            "blk" => blocks.push(
                parts
                    .map(str::trim)
                    .map(|part| block_map.get(part).map(String::as_str).unwrap_or(part))
                    .collect::<Vec<_>>(),
            ),
            "sc" => scripts.push(parts.map(str::trim).collect::<Vec<_>>()),
            "bool" => bools.push(parts.map(str::trim).collect::<Vec<_>>()),
            _ => continue,
        };
    }

    [categories, scripts, blocks, bools]
}
