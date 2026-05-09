/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **UI Component**: Sub-nav controller for Comms and Assets. 
 * Links to Voice Interface (Standups) and Workspace/File System management.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Route prefix mismatch if `/standups` or `/workspaces` path changes, or missing i18n keys for navigation labels.
 * - **Telemetry Link**: Search for `[Asset_Nav]` in UI logs.
 */

import { NavLink } from 'react-router-dom';
import { Mic, FolderOpen } from 'lucide-react';
import { Tooltip } from './ui';

interface AssetNavProps {
    nav_item_class: (props: { isActive: boolean }) => string;
}

import { i18n } from '../i18n';

export const Asset_Nav = ({ nav_item_class }: AssetNavProps): React.ReactElement => {
    return (
        <div className="space-y-1">
            <div className="text-[10px] font-bold text-zinc-600 uppercase tracking-widest mb-2 px-2 hidden lg:block">
                {i18n.t('nav.comms_assets')}
            </div>
            <Tooltip content={i18n.t('nav.voice_interface_standups')} position="right">
                <NavLink to="/standups" className={nav_item_class}>
                    <Mic size={18} />
                    <span className="hidden lg:block">{i18n.t('nav.voice_interface')}</span>
                </NavLink>
            </Tooltip>
            <Tooltip content={i18n.t('nav.file_system_workspaces')} position="right">
                <NavLink to="/workspaces" className={nav_item_class}>
                    <FolderOpen size={18} />
                    <span className="hidden lg:block">{i18n.t('nav.workspaces')}</span>
                </NavLink>
            </Tooltip>
        </div>
    );
};


// Metadata: [Asset_Nav]
