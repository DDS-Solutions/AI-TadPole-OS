/**
 * @file ui/index.ts (Legacy Bridge)
 * @description Bridges legacy components to the centralized UI library.
 */
import * as root_ui from '../../../../src/components/ui';

// Re-export common UI components used by legacy code
export const Tooltip = root_ui.Tooltip;
export const Tw_Dialog = root_ui.Tw_Dialog;
export const Tw_Empty_State = root_ui.Tw_Empty_State;
// Add others as needed
export default root_ui;
