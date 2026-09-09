/**
 * @docs ARCHITECTURE:UI
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Outward / Company_Info_Card_Manager.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { Company_Info_Card_Manager } from './Company_Info_Card_Manager';
import { DEFAULT_INFO_CARDS, type InfoCard } from '../../utils/agent_card_compiler';

describe('Company_Info_Card_Manager Component', () => {
  it('renders title, card count badge, and default info cards', () => {
    const handleCardsChange = vi.fn();
    render(
      <Company_Info_Card_Manager
        cards={DEFAULT_INFO_CARDS}
        onCardsChange={handleCardsChange}
      />
    );

    expect(screen.getByText('Business FAQ & Operating Info Cards')).toBeInTheDocument();
    expect(screen.getByText('Store Hours & Location')).toBeInTheDocument();
    expect(screen.getByText('Return & Exchange Policy')).toBeInTheDocument();
    expect(screen.getByText('Customer Support FAQ')).toBeInTheDocument();
  });

  it('filters cards when category tabs are clicked', () => {
    const handleCardsChange = vi.fn();
    render(
      <Company_Info_Card_Manager
        cards={DEFAULT_INFO_CARDS}
        onCardsChange={handleCardsChange}
      />
    );

    // Click FAQ tab
    const faqTab = screen.getByRole('button', { name: 'FAQ' });
    fireEvent.click(faqTab);

    expect(screen.getByText('Customer Support FAQ')).toBeInTheDocument();
    expect(screen.queryByText('Store Hours & Location')).not.toBeInTheDocument();

    // Click Policies tab
    const policiesTab = screen.getByRole('button', { name: 'Policies' });
    fireEvent.click(policiesTab);

    expect(screen.getByText('Return & Exchange Policy')).toBeInTheDocument();
    expect(screen.queryByText('Customer Support FAQ')).not.toBeInTheDocument();
  });

  it('opens modal and allows creating a new knowledge card', () => {
    let currentCards: InfoCard[] = [...DEFAULT_INFO_CARDS];
    const handleCardsChange = vi.fn((newCards: InfoCard[]) => {
      currentCards = newCards;
    });

    const { rerender } = render(
      <Company_Info_Card_Manager
        cards={currentCards}
        onCardsChange={handleCardsChange}
      />
    );

    // Click Add Knowledge Card button
    const addButton = screen.getByLabelText('Add Knowledge Card');
    fireEvent.click(addButton);

    expect(screen.getByText('Add New Knowledge Card')).toBeInTheDocument();

    // Fill out title, content, and tags
    const titleInput = screen.getByPlaceholderText('e.g. Return & Refund Policy');
    const contentTextarea = screen.getByPlaceholderText('Enter store hours, policy text, FAQ answers, or custom instructions...');
    const tagsInput = screen.getByPlaceholderText('e.g. returns, refunds, policy');

    fireEvent.change(titleInput, { target: { value: 'Warranty Guarantee Card' } });
    fireEvent.change(contentTextarea, { target: { value: 'All power tools come with a 2-year manufacturer warranty.' } });
    fireEvent.change(tagsInput, { target: { value: 'warranty, tools, coverage' } });

    // Submit modal form
    const publishButton = screen.getByText('Publish Card');
    fireEvent.click(publishButton);

    expect(handleCardsChange).toHaveBeenCalled();
    expect(currentCards.length).toBe(DEFAULT_INFO_CARDS.length + 1);
    expect(currentCards[0].title).toBe('Warranty Guarantee Card');

    // Rerender with updated cards
    rerender(
      <Company_Info_Card_Manager
        cards={currentCards}
        onCardsChange={handleCardsChange}
      />
    );

    expect(screen.getByText('Warranty Guarantee Card')).toBeInTheDocument();
  });

  it('allows editing an existing knowledge card', () => {
    let currentCards: InfoCard[] = [...DEFAULT_INFO_CARDS];
    const handleCardsChange = vi.fn((newCards: InfoCard[]) => {
      currentCards = newCards;
    });

    render(
      <Company_Info_Card_Manager
        cards={currentCards}
        onCardsChange={handleCardsChange}
      />
    );

    // Click Edit on the first card
    const editButton = screen.getByLabelText('Edit Knowledge Card Store Hours & Location');
    fireEvent.click(editButton);

    expect(screen.getByText('Edit Knowledge Card')).toBeInTheDocument();

    const titleInput = screen.getByPlaceholderText('e.g. Return & Refund Policy');
    fireEvent.change(titleInput, { target: { value: 'Updated Store Hours & Location' } });

    const saveButton = screen.getByText('Save Changes');
    fireEvent.click(saveButton);

    expect(handleCardsChange).toHaveBeenCalled();
    expect(currentCards.find((c) => c.id === 'info-hours-loc')?.title).toBe('Updated Store Hours & Location');
  });

  it('prevents submitting form when title or content is empty', () => {
    const handleCardsChange = vi.fn();
    render(
      <Company_Info_Card_Manager
        cards={DEFAULT_INFO_CARDS}
        onCardsChange={handleCardsChange}
      />
    );

    fireEvent.click(screen.getByLabelText('Add Knowledge Card'));
    fireEvent.click(screen.getByText('Publish Card'));

    expect(handleCardsChange).not.toHaveBeenCalled();
  });

  it('shows empty state message when cards array is empty', () => {
    render(
      <Company_Info_Card_Manager
        cards={[]}
        onCardsChange={vi.fn()}
      />
    );

    expect(screen.getByText("No knowledge cards found under 'all' category.")).toBeInTheDocument();
    expect(screen.getByText('+ Create your first card')).toBeInTheDocument();
  });

  it('shows PII Warning when credit card pattern is typed into modal form content', () => {
    render(
      <Company_Info_Card_Manager
        cards={DEFAULT_INFO_CARDS}
        onCardsChange={vi.fn()}
      />
    );

    // Open Modal
    fireEvent.click(screen.getByLabelText('Add Knowledge Card'));

    const contentTextarea = screen.getByPlaceholderText('Enter store hours, policy text, FAQ answers, or custom instructions...');
    fireEvent.change(contentTextarea, { target: { value: 'Call 4532 0123 4567 8910 for payment processing' } });

    expect(screen.getByText('PII Warning Detected')).toBeInTheDocument();
    expect(screen.getByText('• Potential Credit Card number detected in text.')).toBeInTheDocument();
  });

  it('triggers delete confirmation dialog and calls onCardsChange when confirmed', () => {
    let currentCards: InfoCard[] = [...DEFAULT_INFO_CARDS];
    const handleCardsChange = vi.fn((newCards: InfoCard[]) => {
      currentCards = newCards;
    });

    render(
      <Company_Info_Card_Manager
        cards={currentCards}
        onCardsChange={handleCardsChange}
      />
    );

    // Click delete on first card
    const deleteButton = screen.getByLabelText('Delete Knowledge Card Store Hours & Location');
    fireEvent.click(deleteButton);

    // Verify Confirm_Dialog popover appears
    expect(screen.getByText('Delete Knowledge Card')).toBeInTheDocument();
    expect(screen.getByText('Are you sure you want to delete this business knowledge card? This will remove the skill from published A2A cards.')).toBeInTheDocument();

    // Confirm deletion
    const confirmDeleteBtn = screen.getByRole('button', { name: 'Delete Card' });
    fireEvent.click(confirmDeleteBtn);

    expect(handleCardsChange).toHaveBeenCalled();
    expect(currentCards.length).toBe(DEFAULT_INFO_CARDS.length - 1);
    expect(currentCards.find((c) => c.id === 'info-hours-loc')).toBeUndefined();
  });
});
