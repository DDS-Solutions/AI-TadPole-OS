/**
 * @docs ARCHITECTURE:UI
 * @docs OPERATIONS_MANUAL:OutwardGateway
 *
 * ### AI Assist Note
 * Unit tests for agent_card_compiler.ts utility functions.
 * Verifies A2A skill compilation, Markdown context delimiters, Luhn algorithm validation, and PII scanning.
 *
 * ### 🔍 Debugging & Observability
 * Traceability via `execution/parity_guard.py`.
 */

import { describe, it, expect } from 'vitest';
import {
  compileInfoCardsToSkills,
  compileInfoCardsToMarkdown,
  isLuhnValid,
  scanForPii,
  DEFAULT_INFO_CARDS,
  type InfoCard,
} from './agent_card_compiler';

describe('agent_card_compiler Utility', () => {
  it('compiles InfoCards to A2aSkills with clean card_ prefixes', () => {
    const cards: InfoCard[] = [
      {
        id: 'hours-1',
        title: 'Operating Hours',
        content: 'Open Mon-Fri 9-5',
        category: 'operating_info',
        tags: ['hours'],
      },
      {
        id: 'card_already_prefixed',
        title: 'Prefixed Card',
        content: 'Already has card_ prefix',
        category: 'custom',
        tags: [],
      },
    ];

    const skills = compileInfoCardsToSkills(cards);

    expect(skills.length).toBe(3); // 1 base catalog_search skill + 2 card skills
    expect(skills[0].id).toBe('catalog_search');
    expect(skills[1].id).toBe('card_hours-1');
    expect(skills[2].id).toBe('card_already_prefixed'); // Does not double-prefix
    expect(skills[2].tags).toEqual(['custom']); // Fallback to category when tags are empty
  });

  it('compiles InfoCards to Markdown bounded by BUSINESS_KNOWLEDGE context delimiters', () => {
    const md = compileInfoCardsToMarkdown([...DEFAULT_INFO_CARDS]);

    expect(md).toContain('<<< BUSINESS_KNOWLEDGE_START >>>');
    expect(md).toContain('<<< BUSINESS_KNOWLEDGE_END >>>');
    expect(md).toContain('#### [OPERATING_INFO] Store Hours & Location');
    expect(md).toContain('#### [POLICIES] Return & Exchange Policy');
  });

  it('validates credit card numbers with Luhn Algorithm Mod 10 check', () => {
    // Valid Visa card
    expect(isLuhnValid('4532 0123 4567 8910')).toBe(true);

    // Invalid number (e.g. order tracking number)
    expect(isLuhnValid('4532 0123 4567 8919')).toBe(false);

    // Short tracking string
    expect(isLuhnValid('12345')).toBe(false);
  });

  it('scans text and detects PII warnings for credit cards, SSNs, and private keys', () => {
    // Valid CC match
    const ccWarnings = scanForPii('Please charge card 4532-0123-4567-8910');
    expect(ccWarnings.length).toBe(1);
    expect(ccWarnings[0]).toContain('Credit Card');

    // False positive (invalid Luhn order ID) should NOT trigger warning
    const invalidCcWarnings = scanForPii('Order ID: 4532-0123-4567-8919');
    expect(invalidCcWarnings.length).toBe(0);

    // SSN match
    const ssnWarnings = scanForPii('My SSN is 123-45-6789');
    expect(ssnWarnings.length).toBe(1);
    expect(ssnWarnings[0]).toContain('Social Security Number');

    // Private key match
    const keyWarnings = scanForPii('-----BEGIN PRIVATE KEY-----');
    expect(keyWarnings.length).toBe(1);
    expect(keyWarnings[0]).toContain('Private cryptographic key');
  });
});
