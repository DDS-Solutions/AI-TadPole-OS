import { NavLink } from 'react-router-dom';
import { Mic, FolderOpen } from 'lucide-react';
import { Tooltip } from './ui';

interface AssetNavProps {
    navItemClass: (props: { isActive: boolean }) => string;
}

import { i18n } from '../i18n';

interface AssetNavProps {
    navItemClass: (props: { isActive: boolean }) => string;
}

export const Asset_Nav = ({ navItemClass }: AssetNavProps): React.ReactElement => {
    return (
        <div className="space-y-1">
            <div className="text-[10px] font-bold text-zinc-600 uppercase tracking-widest mb-2 px-2 hidden lg:block">
                {i18n.t('nav.comms_assets')}
            </div>
            <Tooltip content={i18n.t('nav.voice_interface_standups')} position="right">
                <NavLink to="/standups" className={navItemClass}>
                    <Mic size={18} />
                    <span className="hidden lg:block">{i18n.t('nav.voice_interface')}</span>
                </NavLink>
            </Tooltip>
            <Tooltip content={i18n.t('nav.file_system_workspaces')} position="right">
                <NavLink to="/workspaces" className={navItemClass}>
                    <FolderOpen size={18} />
                    <span className="hidden lg:block">{i18n.t('nav.workspaces')}</span>
                </NavLink>
            </Tooltip>
        </div>
    );
};
