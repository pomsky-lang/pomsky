export class PomskyDiagnostic {
  /**
   * @param {string} message
   * @param {string?} help
   * @param {[number, number]} range
   */
  constructor(message, help, range) {
    this.message = message;
    this.help = help;
    this.range = [range[0], range[1]];
  }
}

export class PomskyError extends Error {
  /**
   * @param {string} message
   * @param {PomskyDiagnostic[]} diagnostics
   */
  constructor(message, diagnostics) {
    super(message);
    this.diagnostics = diagnostics;
  }
}

export class PomskyResult {
  /**
   * @param {string} output
   * @param {PomskyDiagnostic[]} warnings
   */
  constructor(output, warnings) {
    this.output = output;
    this.warnings = warnings;
  }
}
