/**
 * @docs ARCHITECTURE:UI
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Outward / Company_Info_Card_Manager
 * - **Primary Entrypoints**: `Company_Info_Card_Manager`, `CompanyInfoCardManagerProps`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import React, { useState, useMemo, useEffect } from 'react';
import { BookOpen, Plus, Edit2, Trash2, X, AlertTriangle, Tag, HelpCircle, Clock, FileText, Sparkles } from 'lucide-react';
import { Confirm_Dialog, Tooltip } from '../ui';
import { type InfoCard, type InfoCardCategory, scanForPii } from '../../utils/agent_card_compiler';

export interface CompanyInfoCardManagerProps {
  cards: InfoCard[];
  onCardsChange: (newCards: InfoCard[]) => void;
}

interface CardFormData {
  title: string;
  category: InfoCardCategory;
  content: string;
  tags: string;
}

const DEFAULT_FORM_DATA: CardFormData = {
  title: '',
  category: 'faq',
  content: '',
  tags: '',
};

export const Company_Info_Card_Manager: React.FC<CompanyInfoCardManagerProps> = ({
  cards,
  onCardsChange,
}) => {
  const [activeTab, setActiveTab] = useState<InfoCardCategory | 'all'>('all');
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [editingCardId, setEditingCardId] = useState<string | null>(null);
  const [deletingCardId, setDeletingCardId] = useState<string | null>(null);

  // Consolidated Form State
  const [formData, setFormData] = useState<CardFormData>(DEFAULT_FORM_DATA);

  // Helper to update individual form fields
  const updateFormField = <K extends keyof CardFormData>(field: K, value: CardFormData[K]) => {
    setFormData((prev) => ({ ...prev, [field]: value }));
  };

  // Open modal for new card creation
  const handleOpenNewModal = () => {
    setEditingCardId(null);
    setFormData(DEFAULT_FORM_DATA);
    setIsModalOpen(true);
  };

  // Open modal to edit existing card
  const handleOpenEditModal = (card: InfoCard) => {
    setEditingCardId(card.id);
    setFormData({
      title: card.title,
      category: card.category,
      content: card.content,
      tags: card.tags.join(', '),
    });
    setIsModalOpen(true);
  };

  // Close modal handler
  const handleCloseModal = () => {
    setIsModalOpen(false);
  };

  // Escape key handler for accessible modal dismissal
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && isModalOpen) {
        handleCloseModal();
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [isModalOpen]);

  // Save card (Create or Update)
  const handleSaveCard = (e: React.FormEvent) => {
    e.preventDefault();
    if (!formData.title.trim() || !formData.content.trim()) return;

    const parsedTags = formData.tags
      .split(',')
      .map((t) => t.trim().toLowerCase())
      .filter(Boolean);

    if (editingCardId) {
      // Update existing card
      const updated = cards.map((card) =>
        card.id === editingCardId
          ? {
              ...card,
              title: formData.title.trim(),
              category: formData.category,
              content: formData.content.trim(),
              tags: parsedTags,
              updatedAt: new Date().toISOString(),
            }
          : card
      );
      onCardsChange(updated);
    } else {
      // Create new card with collision-resistant UUID
      const cardId = typeof crypto !== 'undefined' && crypto.randomUUID ? crypto.randomUUID() : `${Date.now()}`;
      const newCard: InfoCard = {
        id: `info-${cardId}`,
        title: formData.title.trim(),
        category: formData.category,
        content: formData.content.trim(),
        tags: parsedTags,
        updatedAt: new Date().toISOString(),
      };
      onCardsChange([newCard, ...cards]);
    }

    handleCloseModal();
  };

  // Delete card after confirmation
  const handleConfirmDelete = () => {
    if (!deletingCardId) return;
    const filtered = cards.filter((c) => c.id !== deletingCardId);
    onCardsChange(filtered);
    setDeletingCardId(null);
  };

  // Filter cards by category tab
  const filteredCards = activeTab === 'all'
    ? cards
    : cards.filter((c) => c.category === activeTab);

  // Memoized PII scan to eliminate typing latency on large forms
  const piiWarnings = useMemo(() => scanForPii(formData.content), [formData.content]);

  const getCategoryBadgeClass = (category: InfoCardCategory) => {
    switch (category) {
      case 'faq':
        return 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20';
      case 'operating_info':
        return 'bg-cyan-500/10 text-cyan-400 border-cyan-500/20';
      case 'policies':
        return 'bg-amber-500/10 text-amber-400 border-amber-500/20';
      case 'custom':
        return 'bg-purple-500/10 text-purple-400 border-purple-500/20';
    }
  };

  const getCategoryIcon = (category: InfoCardCategory) => {
    switch (category) {
      case 'faq':
        return <HelpCircle className="w-3.5 h-3.5" />;
      case 'operating_info':
        return <Clock className="w-3.5 h-3.5" />;
      case 'policies':
        return <FileText className="w-3.5 h-3.5" />;
      case 'custom':
        return <Sparkles className="w-3.5 h-3.5" />;
    }
  };

  return (
    <div className="bg-zinc-900/60 backdrop-blur-xl border border-white/5 rounded-2xl p-6 space-y-5 shadow-2xl relative overflow-hidden">
      <div className="absolute top-0 left-0 w-full h-px bg-gradient-to-r from-transparent via-emerald-500/30 to-transparent" />

      {/* Header Container */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3 border-b border-zinc-800/80 pb-4">
        <div>
          <h2 className="text-base font-semibold text-zinc-200 flex items-center gap-2 tracking-tight">
            <BookOpen className="w-5 h-5 text-emerald-400 shrink-0" />
            Business FAQ & Operating Info Cards
            <span className="px-2 py-0.5 bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 rounded-full text-xs font-mono">
              {cards.length} {cards.length === 1 ? 'Card' : 'Cards'}
            </span>
          </h2>
          <p className="text-xs text-zinc-400 mt-1">
            Manage company knowledge cards. Automatically compiled into agent-readable A2A Skills & gemma4 context.
          </p>
        </div>

        <Tooltip content="Add a new business FAQ, policy, operating info, or custom card for agent retrieval." position="top">
          <button
            onClick={handleOpenNewModal}
            aria-label="Add Knowledge Card"
            className="flex items-center gap-2 px-3.5 py-2 bg-emerald-500/10 hover:bg-emerald-500/20 border border-emerald-500/30 text-emerald-400 rounded-xl text-xs font-medium transition-all group cursor-pointer shrink-0 active:scale-[0.98]"
          >
            <Plus className="w-4 h-4 group-hover:scale-110 transition-transform" />
            <span>Add Knowledge Card</span>
          </button>
        </Tooltip>
      </div>

      {/* Category Filter Tabs */}
      <div className="flex items-center gap-1.5 overflow-x-auto pb-1 custom-scrollbar text-xs font-mono">
        {[
          { id: 'all', label: 'All Cards' },
          { id: 'faq', label: 'FAQ' },
          { id: 'operating_info', label: 'Operating Info' },
          { id: 'policies', label: 'Policies' },
          { id: 'custom', label: 'Custom' },
        ].map((tab) => (
          <button
            key={tab.id}
            onClick={() => setActiveTab(tab.id as InfoCardCategory | 'all')}
            className={`px-3 py-1.5 rounded-lg border transition-all cursor-pointer whitespace-nowrap ${
              activeTab === tab.id
                ? 'bg-emerald-500/20 text-emerald-400 border-emerald-500/40 font-semibold'
                : 'bg-zinc-950/30 text-zinc-400 border-zinc-800 hover:text-zinc-200 hover:border-zinc-700'
            }`}
          >
            {tab.label}
          </button>
        ))}
      </div>

      {/* Flexible Card Grid Container */}
      <div className="space-y-3 max-h-[60vh] overflow-y-auto custom-scrollbar pr-1">
        {filteredCards.length === 0 ? (
          <div className="p-6 text-center bg-zinc-950/20 rounded-xl border border-zinc-800/40 space-y-2">
            <p className="text-xs text-zinc-400 italic">
              No knowledge cards found under '{activeTab}' category.
            </p>
            <button
              onClick={handleOpenNewModal}
              className="text-xs text-emerald-400 hover:underline cursor-pointer"
            >
              + Create your first card
            </button>
          </div>
        ) : (
          filteredCards.map((card) => (
            <div
              key={card.id}
              className="p-4 bg-zinc-950/40 rounded-xl border border-zinc-800/80 hover:border-zinc-700 transition-all flex flex-col justify-between gap-2.5 group"
            >
              <div className="flex items-start justify-between gap-3">
                <div className="space-y-1 min-w-0 flex-1">
                  <div className="flex items-center gap-2 flex-wrap">
                    <h3 className="text-sm font-semibold text-zinc-100 truncate">
                      {card.title}
                    </h3>
                    <span
                      className={`inline-flex items-center gap-1 px-2 py-0.5 rounded-md text-[10px] font-mono border ${getCategoryBadgeClass(
                        card.category
                      )}`}
                    >
                      {getCategoryIcon(card.category)}
                      <span className="uppercase">{card.category.replace('_', ' ')}</span>
                    </span>
                  </div>
                  <p className="text-xs text-zinc-300 leading-relaxed font-sans">
                    {card.content}
                  </p>
                </div>

                {/* Card Action Toolbar */}
                <div className="flex items-center gap-1 shrink-0 opacity-80 group-hover:opacity-100 transition-opacity">
                  <button
                    onClick={() => handleOpenEditModal(card)}
                    aria-label={`Edit Knowledge Card ${card.title}`}
                    title="Edit Card"
                    className="p-1.5 hover:bg-zinc-800 text-zinc-400 hover:text-emerald-400 rounded-lg transition-colors cursor-pointer"
                  >
                    <Edit2 className="w-3.5 h-3.5" />
                  </button>
                  <button
                    onClick={() => setDeletingCardId(card.id)}
                    aria-label={`Delete Knowledge Card ${card.title}`}
                    title="Delete Card"
                    className="p-1.5 hover:bg-zinc-800 text-zinc-400 hover:text-red-400 rounded-lg transition-colors cursor-pointer"
                  >
                    <Trash2 className="w-3.5 h-3.5" />
                  </button>
                </div>
              </div>

              {/* Tags Footer */}
              {card.tags.length > 0 && (
                <div className="flex items-center gap-1.5 flex-wrap pt-1 border-t border-zinc-800/40">
                  <Tag className="w-3 h-3 text-zinc-500 shrink-0" />
                  {card.tags.map((tag) => (
                    <span
                      key={tag}
                      className="px-2 py-0.5 bg-zinc-800/60 text-zinc-400 rounded text-[10px] font-mono border border-zinc-700/40"
                    >
                      #{tag}
                    </span>
                  ))}
                </div>
              )}
            </div>
          ))
        )}
      </div>

      {/* Modal Dialog for Add / Edit */}
      {isModalOpen && (
        <div
          className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-zinc-950/70 backdrop-blur-sm animate-in fade-in duration-200"
          onClick={(e) => {
            if (e.target === e.currentTarget) handleCloseModal();
          }}
        >
          <div className="w-full max-w-lg bg-zinc-900 border border-zinc-800 rounded-2xl p-6 space-y-5 shadow-2xl relative font-sans">
            <div className="flex items-center justify-between border-b border-zinc-800 pb-3">
              <h3 className="text-base font-bold text-zinc-100 flex items-center gap-2">
                <BookOpen className="w-5 h-5 text-emerald-400" />
                {editingCardId ? 'Edit Knowledge Card' : 'Add New Knowledge Card'}
              </h3>
              <button
                onClick={handleCloseModal}
                className="p-1 hover:bg-zinc-800 text-zinc-400 hover:text-zinc-200 rounded-lg transition-colors cursor-pointer"
                aria-label="Close Modal"
              >
                <X className="w-4 h-4" />
              </button>
            </div>

            <form onSubmit={handleSaveCard} className="space-y-4">
              <div>
                <label className="block text-xs font-semibold text-zinc-400 uppercase tracking-wider mb-1">
                  Card Title / Question
                </label>
                <input
                  type="text"
                  required
                  value={formData.title}
                  onChange={(e) => updateFormField('title', e.target.value)}
                  placeholder="e.g. Return & Refund Policy"
                  className="w-full px-3.5 py-2.5 bg-zinc-950/40 border border-zinc-800 rounded-xl text-sm text-zinc-100 focus:outline-none focus:border-emerald-500/50 focus:ring-1 focus:ring-emerald-500/30 transition-all font-sans"
                />
              </div>

              <div className="grid grid-cols-2 gap-3">
                <div>
                  <label className="block text-xs font-semibold text-zinc-400 uppercase tracking-wider mb-1">
                    Category
                  </label>
                  <select
                    value={formData.category}
                    onChange={(e) => updateFormField('category', e.target.value as InfoCardCategory)}
                    className="w-full px-3 py-2.5 bg-zinc-950/40 border border-zinc-800 rounded-xl text-xs text-zinc-100 focus:outline-none focus:border-emerald-500/50 cursor-pointer font-mono"
                  >
                    <option value="faq">FAQ</option>
                    <option value="operating_info">Operating Info</option>
                    <option value="policies">Policies</option>
                    <option value="custom">Custom</option>
                  </select>
                </div>

                <div>
                  <label className="block text-xs font-semibold text-zinc-400 uppercase tracking-wider mb-1">
                    Tags (Comma-separated)
                  </label>
                  <input
                    type="text"
                    value={formData.tags}
                    onChange={(e) => updateFormField('tags', e.target.value)}
                    placeholder="e.g. returns, refunds, policy"
                    className="w-full px-3 py-2.5 bg-zinc-950/40 border border-zinc-800 rounded-xl text-xs text-zinc-100 focus:outline-none focus:border-emerald-500/50 transition-all font-mono"
                  />
                </div>
              </div>

              <div>
                <label className="block text-xs font-semibold text-zinc-400 uppercase tracking-wider mb-1">
                  Content / Answer (Agent Knowledge)
                </label>
                <textarea
                  required
                  rows={4}
                  value={formData.content}
                  onChange={(e) => updateFormField('content', e.target.value)}
                  placeholder="Enter store hours, policy text, FAQ answers, or custom instructions..."
                  className="w-full px-3.5 py-2.5 bg-zinc-950/40 border border-zinc-800 rounded-xl text-sm text-zinc-100 focus:outline-none focus:border-emerald-500/50 focus:ring-1 focus:ring-emerald-500/30 transition-all font-sans custom-scrollbar"
                />
              </div>

              {/* Memoized PII Sanitization Warning Banner */}
              {piiWarnings.length > 0 && (
                <div className="p-3 bg-amber-500/10 border border-amber-500/30 rounded-xl text-xs text-amber-300 space-y-1 font-mono animate-in fade-in duration-200">
                  <div className="flex items-center gap-2 font-bold text-amber-400">
                    <AlertTriangle className="w-4 h-4 shrink-0" />
                    <span>PII Warning Detected</span>
                  </div>
                  {piiWarnings.map((warn, i) => (
                    <p key={i} className="text-[11px] leading-tight">
                      • {warn}
                    </p>
                  ))}
                </div>
              )}

              <div className="flex items-center justify-end gap-3 pt-2 border-t border-zinc-800">
                <button
                  type="button"
                  onClick={handleCloseModal}
                  className="px-4 py-2 bg-zinc-950/40 hover:bg-zinc-800 border border-zinc-800 text-zinc-400 hover:text-zinc-200 rounded-xl text-xs font-medium transition-all cursor-pointer"
                >
                  Cancel
                </button>
                <button
                  type="submit"
                  className="px-4 py-2 bg-emerald-500/20 hover:bg-emerald-500/30 border border-emerald-500/40 text-emerald-400 rounded-xl text-xs font-semibold transition-all cursor-pointer active:scale-[0.98]"
                >
                  {editingCardId ? 'Save Changes' : 'Publish Card'}
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

      {/* Delete Safeguard Confirmation Modal */}
      <Confirm_Dialog
        is_open={!!deletingCardId}
        title="Delete Knowledge Card"
        message="Are you sure you want to delete this business knowledge card? This will remove the skill from published A2A cards."
        confirm_label="Delete Card"
        cancel_label="Keep Card"
        variant="danger"
        on_confirm={handleConfirmDelete}
        on_cancel={() => setDeletingCardId(null)}
      />
    </div>
  );
};
