/**
 * @docs ARCHITECTURE:UI
 * @docs OPERATIONS_MANUAL:OutwardGateway
 *
 * ### AI Assist Note
 * Outward A2A Agent Card Preview Component. Renders published agent-card.json
 * capabilities for customer-facing gemma4:e4b local model agent following design.md specs.
 *
 * ### 🔍 Debugging & Observability
 * Traceability via `execution/parity_guard.py`.
 */

import React, { useState } from 'react';
import { ShieldCheck, Bot, Cpu, Tag, Globe, CheckCircle2, Copy, Check } from 'lucide-react';

export type ModelProfile = 'gemma4:e4b' | 'gemma4:e8b' | 'gemma4:full';

export interface A2aSkill {
  id: string;
  name: string;
  description: string;
  tags: string[];
}

export interface A2aAgentCardProps {
  name?: string;
  version?: string;
  description?: string;
  url?: string;
  modelProfile?: ModelProfile | string;
  skills?: A2aSkill[];
}

export const A2A_Card_Preview: React.FC<A2aAgentCardProps> = ({
  name = 'SMB Customer Service Agent',
  version = '1.0.0',
  description = 'Sovereign SMB Customer Service & Catalog Agent powered by AI-Tadpole-OS.',
  url = 'http://localhost:26453/a2a/v1/agent-card.json',
  modelProfile = 'gemma4:e4b',
  skills = [
    {
      id: 'catalog_search',
      name: 'Customer Catalog Search',
      description: 'Search product and service catalog for business inquiries.',
      tags: ['catalog', 'products', 'smb'],
    },
    {
      id: 'business_qa',
      name: 'Business FAQ & Operating Info',
      description: 'Answer store hours, location, and service policy questions.',
      tags: ['faq', 'info'],
    },
  ],
}) => {
  const [copied, setCopied] = useState(false);

  const handleCopyUrl = () => {
    navigator.clipboard.writeText(url);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="w-full max-w-2xl bg-zinc-900/60 backdrop-blur-xl border border-white/5 rounded-2xl p-6 text-zinc-100 font-mono shadow-2xl overflow-hidden flex flex-col justify-between relative">
      <div className="absolute top-0 left-0 w-full h-px bg-gradient-to-r from-transparent via-emerald-500/30 to-transparent" />
      <div>
        {/* Header Container */}
        <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3 border-b border-zinc-800/80 pb-4 mb-4">
          <div className="flex items-center space-x-3 min-w-0">
            <div className="p-2.5 bg-emerald-500/10 text-emerald-400 rounded-xl border border-emerald-500/20 shrink-0">
              <Bot className="w-6 h-6" />
            </div>
            <div className="min-w-0 flex-1">
              <h3 className="text-lg font-bold text-zinc-100 truncate tracking-tight">
                {name}
              </h3>
              <div className="flex items-center gap-2 mt-0.5 truncate">
                <a
                  href={url}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-xs text-zinc-400 hover:text-emerald-400 hover:underline transition-colors flex items-center gap-1 truncate"
                >
                  <Globe className="w-3.5 h-3.5 text-zinc-500 shrink-0" />
                  <span className="truncate">{url}</span>
                </a>
                <button
                  onClick={handleCopyUrl}
                  className="p-1 hover:bg-zinc-800 text-zinc-400 hover:text-zinc-200 rounded transition-colors cursor-pointer shrink-0"
                  aria-label="Copy A2A Agent Card URL"
                  title="Copy URL"
                >
                  {copied ? <Check className="w-3.5 h-3.5 text-emerald-400" /> : <Copy className="w-3.5 h-3.5" />}
                </button>
              </div>
            </div>
          </div>

          {/* Model Profile Badge */}
          <div className="flex items-center space-x-2 text-xs bg-black/40 px-3 py-1.5 rounded-xl border border-zinc-800 shrink-0 self-start sm:self-auto shadow-sm">
            <Cpu className="w-4 h-4 text-cyan-400 shrink-0" />
            <span className="text-zinc-400">Model:</span>
            <span className="font-bold text-cyan-400">{modelProfile}</span>
          </div>
        </div>

        <p className="text-xs sm:text-sm text-zinc-300 mb-6 leading-relaxed bg-black/40 p-3.5 rounded-xl border border-zinc-800/60">
          {description}
        </p>

        <div className="space-y-3">
          <h4 className="text-xs font-semibold text-zinc-400 uppercase tracking-wider flex items-center gap-1.5 font-mono">
            <ShieldCheck className="w-4 h-4 text-emerald-400" />
            Exposed A2A Skills ({skills.length})
          </h4>

          {skills.length === 0 ? (
            <div className="p-4 text-xs text-zinc-500 italic bg-black/30 rounded-xl border border-zinc-800/60 text-center">
              No skills currently exposed in this agent card.
            </div>
          ) : (
            <div className="grid grid-cols-1 gap-3">
              {skills.map((skill) => (
                <div
                  key={skill.id}
                  className="p-4 bg-black/40 rounded-xl border border-zinc-800/60 hover:border-zinc-700/80 transition-all"
                >
                  <div className="flex items-center justify-between mb-1.5">
                    <span className="text-sm font-semibold text-emerald-300 flex items-center gap-1.5">
                      <CheckCircle2 className="w-4 h-4 text-emerald-400 shrink-0" />
                      {skill.name}
                    </span>
                    <span className="text-[10px] text-zinc-500 font-mono bg-zinc-800/50 px-2 py-0.5 rounded border border-zinc-800">ID: {skill.id}</span>
                  </div>
                  <p className="text-xs text-zinc-400 mb-2.5 leading-relaxed">{skill.description}</p>
                  <div className="flex flex-wrap gap-1.5">
                    {skill.tags.map((tag) => (
                      <span
                        key={tag}
                        className="text-[10px] bg-zinc-800/80 text-zinc-300 px-2 py-0.5 rounded-md border border-zinc-700/50 flex items-center gap-1 font-mono"
                      >
                        <Tag className="w-2.5 h-2.5 text-zinc-400" />
                        {tag}
                      </span>
                    ))}
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>

      {/* Footer Bar: Version Badge placed at Bottom Right */}
      <div className="mt-6 pt-3 border-t border-zinc-800/60 flex items-center justify-between text-xs text-zinc-500 font-mono">
        <span className="flex items-center gap-1.5">
          <ShieldCheck className="w-3.5 h-3.5 text-emerald-400 shrink-0" />
          <span>A2A Gateway Spec</span>
        </span>
        <span className="text-xs px-2.5 py-0.5 bg-emerald-950/80 text-emerald-400 border border-emerald-800/80 rounded-full font-mono font-semibold shadow-sm">
          v{version}
        </span>
      </div>
    </div>
  );
};
