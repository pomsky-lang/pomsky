const process = require("process");

const args = process.argv.slice(1);

if (args.length === 2 && args[0] === "supports") {
  process.exit(args[1] === "html" ? 0 : 1);
} else {
  process.stdin.on("data", (data) => {
    const content = JSON.parse(data.toString());
    preprocess(content);
  });
}

function preprocess([_context, book]) {
  for (const section of book.sections) {
    try {
      handleBookItem(section);
    } catch {}
  }

  process.stdout.write(JSON.stringify(book));
}

function handleBookItem({ Chapter }) {
  Chapter.content = Chapter.content.replace(
    /<(rulex|regexp)>(`+)(?!`) ?(.*?) ?(?<!`)\2/g,
    (_all, type, _quote, value) => {
      return `<code class="language-${type}">${escapeHtmlAndCommonMark(
        value
      )}</code>`;
    }
  );
  for (const sub_item of Chapter.sub_items) {
    try {
      handleBookItem(sub_item);
    } catch {}
  }
}

function escapeHtmlAndCommonMark(unsafe) {
  return unsafe.replace(/[&<"'\\]/g, function (m) {
    switch (m) {
      case "&":
        return "&amp;";
      case "<":
        return "&lt;";
      case '"':
        return "&quot;";
      case "'":
        return "&#039;";
      case "\\":
        return "\\\\";
    }
  });
}
