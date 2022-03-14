hljs.registerLanguage("rulex", function (hljs) {
  return {
    name: "Rulex",
    aliases: ["rulex"],
    contains: [
      hljs.HASH_COMMENT_MODE,
      {
        className: "string",
        variants: [
          { begin: /"/, end: /"/ },
          { begin: /'/, end: /'/ },
        ],
      },
      {
        className: "keyword",
        beginKeywords: "greedy lazy range base Grapheme X enable disable %",
      },
      {
        className: "symbol",
        begin: ":\\s*[A-Za-z0-9]*",
      },
      {
        className: "title",
        begin: "\\b[A-Za-z0-9]+\\b|\\.",
      },
      {
        className: "operator",
        begin: "[+\\-*{}!=<>!]+",
      },
      {
        className: "punctuation",
        begin: "[\\[\\]()|,]+",
      },
      {
        className: "number",
        variants: [
          {
            begin: "\\b\\d+\\b",
          },
        ],
      },
    ],
  };
});

hljs.initHighlightingOnLoad();
