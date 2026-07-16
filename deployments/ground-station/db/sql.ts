/**
 * Remove line comments (--) and block comments (/* *\/) from SQL,
 * collapse empty lines, and split into statements by semicolon.
 * This is a simplistic splitter and will not handle semicolons inside strings.
 */
export function parseSqlStatements(sql: string): string[] {
  // Remove block comments
  const noBlock = sql.replace(/\/\*[\s\S]*?\*\//g, "");
  // Remove line comments
  const noLine = noBlock.replace(/^\s*--.*$/gm, "");

  const lines = noLine.split(/\r?\n/);
  const statements: string[] = [];
  let buffer: string[] = [];
  let inTrigger = false;

  const pushBuffer = () => {
    const stmt = buffer.join("\n").trim();
    if (stmt.length > 0) {
      statements.push(stmt);
    }
    buffer = [];
  };

  for (const originalLine of lines) {
    const line = originalLine; // preserve indentation
    const trimmed = line.trim();

    if (!inTrigger && /^CREATE\s+TRIGGER/i.test(trimmed)) {
      inTrigger = true;
      buffer.push(line);
      continue;
    }

    if (inTrigger) {
      buffer.push(line);
      if (/\bEND\s*;\s*$/i.test(trimmed)) {
        inTrigger = false;
        pushBuffer();
      }
      continue;
    }

    if (trimmed.length === 0) {
      // Keep blank lines within a statement for readability
      if (buffer.length > 0) buffer.push(line);
      continue;
    }

    buffer.push(line);
    if (trimmed.endsWith(";")) {
      pushBuffer();
    }
  }

  // Flush any remaining buffer
  pushBuffer();

  return statements;
}

export interface StatementExecutor {
  execute(sql: string): Promise<void>;
}
