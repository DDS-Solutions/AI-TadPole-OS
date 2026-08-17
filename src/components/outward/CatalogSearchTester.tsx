/**
 * @docs ARCHITECTURE:UI
 * @docs OPERATIONS_MANUAL:CustomerCatalog
 * 
 * ### AI Assist Note
 * Live catalog search query tester UI component for real-time item preview.
 * Allows SMB owners to test semantic/fuzzy catalog search queries and verify ingested prices.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Search query filter mismatch or malformed price formatting.
 * - **Telemetry Link**: Search `[CatalogSearchTester]` in UI logs.
 */

import React, { memo } from 'react';
import { Search } from 'lucide-react';
import type { CatalogItem } from '../../utils/csv_parser';

export interface CatalogSearchTesterProps {
  searchQuery: string;
  onSearchChange: (q: string) => void;
  filteredItems: CatalogItem[];
}

export const CatalogSearchTester: React.FC<CatalogSearchTesterProps> = memo(({
  searchQuery,
  onSearchChange,
  filteredItems,
}) => {
  return (
    <div className="bg-zinc-900/60 backdrop-blur-xl border border-white/5 rounded-2xl p-6 space-y-4 shadow-2xl relative overflow-hidden">
      <div className="absolute top-0 left-0 w-full h-px bg-gradient-to-r from-transparent via-cyan-500/30 to-transparent" />
      <div className="flex items-center justify-between">
        <h2 className="text-base font-semibold text-zinc-200 flex items-center gap-2 tracking-tight">
          <Search className="w-5 h-5 text-cyan-400" />
          Live Catalog Search Tester
        </h2>
        <span className="text-xs font-mono text-cyan-400 bg-cyan-500/10 px-2 py-0.5 rounded border border-cyan-500/20">
          {filteredItems.length} {filteredItems.length === 1 ? 'item' : 'items'}
        </span>
      </div>

      <div className="relative">
        <Search className="w-4 h-4 text-zinc-500 absolute left-3 top-3" />
        <input
          type="text"
          value={searchQuery}
          onChange={(e) => onSearchChange(e.target.value)}
          placeholder="Search ingested catalog items by title or category..."
          className="w-full pl-9 pr-3.5 py-2.5 bg-zinc-950/40 border border-zinc-800 rounded-xl text-sm text-zinc-100 placeholder-zinc-500 focus:outline-none focus:border-cyan-500/50 focus:ring-1 focus:ring-cyan-500/30 transition-all"
        />
      </div>

      <div className="max-h-48 overflow-y-auto space-y-2 pr-1 custom-scrollbar">
        {filteredItems.map((item) => (
          <div
            key={item.id}
            className="flex items-center justify-between p-3 bg-zinc-950/30 border border-zinc-800/60 rounded-xl text-xs hover:border-zinc-700/80 transition-colors"
          >
            <div className="flex flex-col min-w-0 pr-2">
              <span className="font-medium text-zinc-200 truncate">{item.title}</span>
              <span className="text-[11px] text-zinc-500 font-mono">{item.category}</span>
            </div>
            <span className="font-mono text-emerald-400 font-semibold shrink-0">{item.price}</span>
          </div>
        ))}
        {filteredItems.length === 0 && (
          <p className="text-xs text-zinc-500 italic text-center py-4">No matching catalog items found for "{searchQuery}".</p>
        )}
      </div>
    </div>
  );
});

CatalogSearchTester.displayName = 'CatalogSearchTester';

// Metadata: [CatalogSearchTester]
