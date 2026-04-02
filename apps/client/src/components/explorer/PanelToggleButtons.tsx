import React from 'react';

interface PanelToggleButtonsProps {
  rightSidebarCollapsed: boolean;
  bottomPanelCollapsed: boolean;
  onToggleRightSidebar: () => void;
  onToggleBottomPanel: () => void;
}

const PanelToggleButtons = React.memo(
  ({
    rightSidebarCollapsed: _rightSidebarCollapsed,
    bottomPanelCollapsed: _bottomPanelCollapsed,
    onToggleRightSidebar: _onToggleRightSidebar,
    onToggleBottomPanel: _onToggleBottomPanel,
  }: PanelToggleButtonsProps) => {
    return null;
  },
);

export default PanelToggleButtons;
