/**
 * @docs ARCHITECTURE:UI
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Utilities / csv_parser
 * - **Primary Entrypoints**: `parseCsvText`, `CatalogItem`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Deterministic internal state integrity and strict interface contract compliance.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

export interface CatalogItem {
  id: string;
  title: string;
  category: string;
  price: string;
}

/**
 * Robust quote-aware CSV parser supporting multiline fields, escaped quotes (""), and backslash quotes (\").
 */
export function parseCsvText(text: string): CatalogItem[] {
  if (!text || !text.trim()) return [];

  const rows: string[][] = [];
  let currentRow: string[] = [];
  let currentCell = '';
  let inQuotes = false;

  for (let i = 0; i < text.length; i++) {
    const char = text[i];
    const nextChar = text[i + 1];

    if (char === '\\' && (nextChar === '"' || nextChar === '\\')) {
      currentCell += nextChar;
      i++;
    } else if (char === '"') {
      if (inQuotes && nextChar === '"') {
        currentCell += '"';
        i++;
      } else {
        inQuotes = !inQuotes;
      }
    } else if (char === ',' && !inQuotes) {
      currentRow.push(currentCell.trim());
      currentCell = '';
    } else if ((char === '\r' || char === '\n') && !inQuotes) {
      if (char === '\r' && nextChar === '\n') i++;
      currentRow.push(currentCell.trim());
      if (currentRow.some((c) => c.length > 0)) rows.push(currentRow);
      currentRow = [];
      currentCell = '';
    } else {
      currentCell += char;
    }
  }

  if (currentCell.length > 0 || currentRow.length > 0) {
    currentRow.push(currentCell.trim());
    if (currentRow.some((c) => c.length > 0)) rows.push(currentRow);
  }

  if (rows.length === 0) return [];

  // If header exists, skip first row for items
  const firstRowIsHeader = rows.length > 1 && isNaN(Number(rows[0][rows[0].length - 1].replace(/[$\s]/g, '')));
  const dataRows = firstRowIsHeader ? rows.slice(1) : rows;

  return dataRows.map((row, idx) => {
    const title = row[0] || `Catalog Item ${idx + 1}`;
    const category = row.length >= 3 ? row[1] : 'Imported';
    const priceRaw = row.length >= 3 ? row[2] : row[1] || '$0.00';
    const price = priceRaw.startsWith('$') ? priceRaw : `$${priceRaw}`;

    return {
      id: `csv-${idx + 1}-${Date.now()}`,
      title,
      category,
      price,
    };
  });
}
