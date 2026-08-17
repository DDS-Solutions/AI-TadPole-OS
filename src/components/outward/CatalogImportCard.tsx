/**
 * @docs ARCHITECTURE:UI
 * @docs OPERATIONS_MANUAL:CustomerCatalog
 * 
 * ### AI Assist Note
 * SMB catalog file import card component with 1-click dropzone buttons and status notifications.
 * Handles CSV and QuickBooks ingestion with an explicit unavailable state for PDF.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Failed file upload handler or inaccessible input ref.
 * - **Telemetry Link**: Search `[CatalogImportCard]` in UI logs.
 */

import React, { memo } from 'react';
import { Upload, FileSpreadsheet, FileText, CheckCircle, X, Loader2 } from 'lucide-react';
import { Tooltip } from '../ui';
import type { IngestionType } from './useCatalogIngestion';

export interface CatalogImportCardProps {
  isProcessing: boolean;
  activeType: IngestionType | null;
  importStatus: string | null;
  onDismissStatus: () => void;
  onCsvClick: () => void;
  onQbClick: () => void;
  onPdfClick: () => void;
  csvInputRef: React.RefObject<HTMLInputElement | null>;
  qbInputRef: React.RefObject<HTMLInputElement | null>;
  pdfInputRef: React.RefObject<HTMLInputElement | null>;
  onCsvFileChange: (e: React.ChangeEvent<HTMLInputElement>) => void;
  onQbFileChange: (e: React.ChangeEvent<HTMLInputElement>) => void;
  onPdfFileChange: (e: React.ChangeEvent<HTMLInputElement>) => void;
}

export const CatalogImportCard: React.FC<CatalogImportCardProps> = memo(({
  isProcessing,
  activeType,
  importStatus,
  onDismissStatus,
  onCsvClick,
  onQbClick,
  onPdfClick,
  csvInputRef,
  qbInputRef,
  pdfInputRef,
  onCsvFileChange,
  onQbFileChange,
  onPdfFileChange,
}) => {
  return (
    <div className="bg-zinc-900/60 backdrop-blur-xl border border-white/5 rounded-2xl p-6 space-y-4 shadow-2xl relative overflow-hidden">
      {/* Hidden File Inputs */}
      <input
        type="file"
        ref={csvInputRef}
        onChange={onCsvFileChange}
        accept=".csv,text/csv"
        className="hidden"
        aria-label="CSV Catalog File Input"
      />
      <input
        type="file"
        ref={qbInputRef}
        onChange={onQbFileChange}
        accept=".json,application/json"
        className="hidden"
        aria-label="QuickBooks File Input"
      />
      <input
        type="file"
        ref={pdfInputRef}
        onChange={onPdfFileChange}
        accept=".pdf,application/pdf"
        className="hidden"
        aria-label="PDF Pricelist File Input"
      />

      <div className="absolute top-0 left-0 w-full h-px bg-gradient-to-r from-transparent via-emerald-500/30 to-transparent" />
      <h2 className="text-base font-semibold text-zinc-200 flex items-center gap-2 tracking-tight">
        <Upload className="w-5 h-5 text-emerald-400" />
        1-Click SMB Data Ingestion
      </h2>
      <p className="text-xs text-zinc-400 leading-relaxed">
        Upload product listings, invoice history, or pricelists. Data is parsed locally and converted into sanitized Markdown contexts for local model retrieval.
      </p>

      <div className="grid grid-cols-3 gap-3 pt-2">
        <Tooltip content="Ingest product lists from CSV spreadsheet files with quote-aware line parsing." position="top">
          <button
            disabled={isProcessing}
            onClick={onCsvClick}
            aria-label="Upload CSV Catalog File"
            className="w-full flex flex-col items-center justify-center p-4 bg-zinc-950/40 border border-zinc-800/80 hover:border-emerald-500/40 hover:bg-emerald-500/5 rounded-xl text-zinc-300 hover:text-emerald-400 transition-all group cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed active:scale-[0.98]"
          >
            {isProcessing && activeType === 'csv' ? (
              <Loader2 className="w-6 h-6 mb-2 text-emerald-400 animate-spin" />
            ) : (
              <FileSpreadsheet className="w-6 h-6 mb-2 text-emerald-400 group-hover:scale-110 transition-transform" />
            )}
            <span className="text-xs font-mono font-medium">CSV Catalog</span>
          </button>
        </Tooltip>

        <Tooltip content="Import product catalog items and unit pricing from QuickBooks JSON exports." position="top">
          <button
            disabled={isProcessing}
            onClick={onQbClick}
            aria-label="Upload QuickBooks JSON File"
            className="w-full flex flex-col items-center justify-center p-4 bg-zinc-950/40 border border-zinc-800/80 hover:border-cyan-500/40 hover:bg-cyan-500/5 rounded-xl text-zinc-300 hover:text-cyan-400 transition-all group cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed active:scale-[0.98]"
          >
            {isProcessing && activeType === 'qb' ? (
              <Loader2 className="w-6 h-6 mb-2 text-cyan-400 animate-spin" />
            ) : (
              <FileText className="w-6 h-6 mb-2 text-cyan-400 group-hover:scale-110 transition-transform" />
            )}
            <span className="text-xs font-mono font-medium">QuickBooks</span>
          </button>
        </Tooltip>

        <Tooltip content="PDF ingestion is unavailable. Export the pricelist as CSV or QuickBooks JSON." position="top">
          <button
            disabled
            onClick={onPdfClick}
            aria-label="PDF Pricelist Import Unavailable"
            className="w-full flex flex-col items-center justify-center p-4 bg-zinc-950/40 border border-zinc-800/80 hover:border-amber-500/40 hover:bg-amber-500/5 rounded-xl text-zinc-300 hover:text-amber-400 transition-all group cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed active:scale-[0.98]"
          >
            {isProcessing && activeType === 'pdf' ? (
              <Loader2 className="w-6 h-6 mb-2 text-amber-400 animate-spin" />
            ) : (
              <Upload className="w-6 h-6 mb-2 text-amber-400 group-hover:scale-110 transition-transform" />
            )}
            <span className="text-xs font-mono font-medium">PDF Unavailable</span>
          </button>
        </Tooltip>
      </div>

      {importStatus && (
        <div className="p-3 bg-emerald-950/40 border border-emerald-800/60 rounded-xl text-xs text-emerald-300 flex items-center justify-between font-mono animate-in fade-in duration-300">
          <div className="flex items-center gap-2">
            <CheckCircle className="w-4 h-4 text-emerald-400 flex-shrink-0" />
            <span>{importStatus}</span>
          </div>
          <button
            onClick={onDismissStatus}
            className="p-1 hover:bg-emerald-900/50 rounded text-emerald-400 transition-colors cursor-pointer"
            aria-label="Dismiss status notification"
            title="Dismiss"
          >
            <X className="w-3.5 h-3.5" />
          </button>
        </div>
      )}
    </div>
  );
});

CatalogImportCard.displayName = 'CatalogImportCard';

// Metadata: [CatalogImportCard]
