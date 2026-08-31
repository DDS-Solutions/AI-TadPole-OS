/**
 * @docs ARCHITECTURE:UI
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Utilities / agent_card_compiler
 * - **Primary Entrypoints**: `compileInfoCardsToSkills`, `compileInfoCardsToMarkdown`, `isLuhnValid`, `scanForPii`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Deterministic internal state integrity and strict interface contract compliance.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import type { A2aSkill } from '../components/outward/A2A_Card_Preview';

export type InfoCardCategory = 'faq' | 'operating_info' | 'policies' | 'custom';

export interface InfoCard {
  id: string;
  title: string;
  content: string;
  category: InfoCardCategory;
  tags: string[];
  updatedAt?: string;
}

/**
 * Default initial knowledge cards for SMB Customer Service Agent (Readonly)
 */
export const DEFAULT_INFO_CARDS: readonly InfoCard[] = [
  {
    id: 'info-hours-loc',
    title: 'Store Hours & Location',
    content: 'Main Store: 100 Main St, Suite 400. Hours: Monday - Friday 8:00 AM - 6:00 PM EST, Saturday 9:00 AM - 4:00 PM EST.',
    category: 'operating_info',
    tags: ['hours', 'location', 'contact'],
  },
  {
    id: 'info-faq-returns',
    title: 'Return & Exchange Policy',
    content: 'Full refunds issued within 30 days of purchase with original receipt. Items must be in original packaging and unused.',
    category: 'policies',
    tags: ['returns', 'refunds', 'guarantee'],
  },
  {
    id: 'info-faq-support',
    title: 'Customer Support FAQ',
    content: 'For immediate order tracking or service inquiries, email support@tadpolesmb.com or call our service desk.',
    category: 'faq',
    tags: ['faq', 'support', 'help'],
  },
];

/**
 * Compiles a list of InfoCard items into standard A2A Protocol skills
 */
export function compileInfoCardsToSkills(cards: readonly InfoCard[]): A2aSkill[] {
  const baseSkills: A2aSkill[] = [
    {
      id: 'catalog_search',
      name: 'Customer Catalog Search',
      description: 'Search product and service catalog for business inquiries.',
      tags: ['catalog', 'products', 'smb'],
    },
  ];

  const cardSkills: A2aSkill[] = cards.map((card) => ({
    id: card.id.startsWith('card_') ? card.id : `card_${card.id}`,
    name: card.title,
    description: card.content.length > 120 ? `${card.content.slice(0, 117)}...` : card.content,
    tags: card.tags.length > 0 ? card.tags : [card.category],
  }));

  return [...baseSkills, ...cardSkills];
}

const CONTEXT_MARKER_REGEX = /<{3,}\s*[A-Z_]+(?:_[A-Z_]+)*\s*>{3,}/gi;

/**
 * Sanitizes context strings to prevent prompt-injection breakout of delimitated blocks
 */
function sanitize_context_text(text: string): string {
  if (!text) return '';
  return text.replace(CONTEXT_MARKER_REGEX, '[sanitized_marker]');
}

/**
 * Compiles InfoCard items into clean Markdown bounded by context markers for gemma4 local model inference
 */
export function compileInfoCardsToMarkdown(cards: InfoCard[]): string {
  if (cards.length === 0) {
    return '<<< BUSINESS_KNOWLEDGE_START >>>\n### Business FAQ & Operating Info\nNo business cards registered.\n<<< BUSINESS_KNOWLEDGE_END >>>';
  }

  const lines = ['<<< BUSINESS_KNOWLEDGE_START >>>', '### Business FAQ & Operating Info\n'];

  cards.forEach((card) => {
    const safeTitle = sanitize_context_text(card.title);
    const safeContent = sanitize_context_text(card.content);
    const safeCategory = sanitize_context_text(card.category);
    const safeTags = card.tags.map((t) => sanitize_context_text(t)).join(', ');

    lines.push(`#### [${safeCategory.toUpperCase()}] ${safeTitle}`);
    lines.push(`${safeContent}`);
    if (card.tags.length > 0) {
      lines.push(`*Tags*: ${safeTags}`);
    }
    lines.push('');
  });

  lines.push('<<< BUSINESS_KNOWLEDGE_END >>>');

  return lines.join('\n');
}

/**
 * Validates a numerical string against Luhn's Algorithm (Mod 10 Modulus check)
 */
export function isLuhnValid(val: string): boolean {
  const digits = val.replace(/\D/g, '');
  if (digits.length < 13 || digits.length > 19) return false;

  let sum = 0;
  let shouldDouble = false;

  for (let i = digits.length - 1; i >= 0; i--) {
    let digit = parseInt(digits.charAt(i), 10);
    if (shouldDouble) {
      digit *= 2;
      if (digit > 9) digit -= 9;
    }
    sum += digit;
    shouldDouble = !shouldDouble;
  }

  return sum % 10 === 0;
}

/**
 * Scans input text for sensitive PII (credit cards verified via Luhn check, SSNs, private keys)
 */
export function scanForPii(text: string): string[] {
  if (!text || typeof text !== 'string') return [];
  const warnings: string[] = [];

  // Credit Card Regex Match + Luhn Verification for all matches
  // Covers Visa (4), Mastercard (51-55, 2221-2720), Amex (34/37), Discover (6011, 65, 644-649), Diners Club, JCB
  const ccPattern = /\b(?:4[0-9]{12}(?:[0-9]{3})?|5[1-5][0-9]{14}|2(?:22[1-9]|2[3-9][0-9]|[3-6][0-9]{2}|7[0-1][0-9]|720)[0-9]{12}|3[47][0-9]{13}|6(?:011|5[0-9]{2}|4[4-9][0-9])[0-9]{12}|3(?:0[0-5]|[68][0-9])[0-9]{11}|(?:2131|1800|35\d{3})\d{11}|(?:[0-9]{4}[ -]?){3}[0-9]{4}|[0-9]{4}[ -]?[0-9]{6}[ -]?[0-9]{5})\b/g;

  const matches = text.matchAll(ccPattern);
  let ccFound = false;
  for (const m of matches) {
    if (isLuhnValid(m[0])) {
      ccFound = true;
      break;
    }
  }
  if (ccFound) {
    warnings.push('Potential Credit Card number detected in text.');
  }

  // Social Security Number Pattern (supports hyphens and spaces)
  const ssnPattern = /\b\d{3}[ -]\d{2}[ -]\d{4}\b/;
  if (ssnPattern.test(text)) {
    warnings.push('Potential Social Security Number (SSN) pattern detected.');
  }

  // Private Key Pattern (catches RSA, EC, OPENSSH, DSA, PGP, ENCRYPTED)
  const privateKeyPattern = /-----BEGIN[\w\s-]*PRIVATE KEY/i;
  if (privateKeyPattern.test(text) || text.includes('PRIVATE KEY BLOCK') || text.includes('BEGIN OPENSSH PRIVATE KEY')) {
    warnings.push('Private cryptographic key detected!');
  }

  return warnings;
}
