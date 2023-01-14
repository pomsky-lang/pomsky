export class PomskyDiagnostic {
  /**
   * @param {"error" | "warning"} kind
   * @param {string} code
   * @param {string} message
   * @param {string?} help
   * @param {[number, number]} range
   */
  constructor(kind, code, message, help, range) {
    this.kind = kind
    this.code = code
    this.message = message
    this.help = help
    this.range = [range[0], range[1]]
  }
}

export class PomskyError extends Error {
  /**
   * @param {string} message
   */
  constructor(message) {
    super(message)
  }
}

export class PomskyResult {
  /**
   * @param {string | null} output
   * @param {PomskyDiagnostic[]} diagnostics
   */
  constructor(output, diagnostics) {
    this.output = output
    this.diagnostics = diagnostics
  }
}
