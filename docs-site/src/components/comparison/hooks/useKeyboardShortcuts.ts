/**
 * Keyboard Shortcuts Hook
 * Provides comprehensive keyboard navigation for the comparison app
 */

import { useEffect, useCallback, useRef } from 'react';
import type { TabType } from '../types/comparison';

interface KeyboardShortcutsConfig {
  onTabChange: (tab: TabType) => void;
  onToggleTheme?: () => void;
  onSearch?: () => void;
  onExport?: () => void;
  onHelp?: () => void;
  currentTab: TabType;
  disabled?: boolean;
}

interface ShortcutAction {
  key: string;
  ctrlKey?: boolean;
  altKey?: boolean;
  shiftKey?: boolean;
  metaKey?: boolean;
  description: string;
  action: () => void;
}

export function useKeyboardShortcuts({
  onTabChange,
  onToggleTheme,
  onSearch,
  onExport,
  onHelp,
  currentTab,
  disabled = false,
}: KeyboardShortcutsConfig) {
  const shortcutsRef = useRef<ShortcutAction[]>([]);
  const helpModalRef = useRef<boolean>(false);

  // Define keyboard shortcuts
  const shortcuts: ShortcutAction[] = [
    // Tab navigation
    {
      key: '1',
      altKey: true,
      description: 'Switch to Overview tab',
      action: () => onTabChange('overview'),
    },
    {
      key: '2',
      altKey: true,
      description: 'Switch to Implementations tab',
      action: () => onTabChange('implementations'),
    },
    {
      key: '3',
      altKey: true,
      description: 'Switch to Details tab',
      action: () => onTabChange('details'),
    },
    {
      key: '4',
      altKey: true,
      description: 'Switch to Analysis tab',
      action: () => onTabChange('analysis'),
    },

    // Navigation within tabs
    {
      key: 'ArrowLeft',
      altKey: true,
      description: 'Previous tab',
      action: () => {
        const tabs: TabType[] = ['overview', 'implementations', 'details', 'analysis'];
        const currentIndex = tabs.indexOf(currentTab);
        const prevIndex = currentIndex > 0 ? currentIndex - 1 : tabs.length - 1;
        onTabChange(tabs[prevIndex]);
      },
    },
    {
      key: 'ArrowRight',
      altKey: true,
      description: 'Next tab',
      action: () => {
        const tabs: TabType[] = ['overview', 'implementations', 'details', 'analysis'];
        const currentIndex = tabs.indexOf(currentTab);
        const nextIndex = currentIndex < tabs.length - 1 ? currentIndex + 1 : 0;
        onTabChange(tabs[nextIndex]);
      },
    },

    // Theme toggle
    ...(onToggleTheme ? [{
      key: 't',
      ctrlKey: true,
      description: 'Toggle dark/light theme',
      action: onToggleTheme,
    }] : []),

    // Search functionality
    ...(onSearch ? [{
      key: 'f',
      ctrlKey: true,
      description: 'Focus search input',
      action: (e?: KeyboardEvent) => {
        e?.preventDefault();
        onSearch();
      },
    }] : []),

    // Export functionality
    ...(onExport ? [{
      key: 'e',
      ctrlKey: true,
      shiftKey: true,
      description: 'Export current data',
      action: (e?: KeyboardEvent) => {
        e?.preventDefault();
        onExport();
      },
    }] : []),

    // Help
    {
      key: '?',
      shiftKey: true,
      description: 'Show keyboard shortcuts help',
      action: () => {
        if (onHelp) {
          onHelp();
        } else {
          showDefaultHelp();
        }
      },
    },
    {
      key: 'F1',
      description: 'Show keyboard shortcuts help',
      action: () => {
        if (onHelp) {
          onHelp();
        } else {
          showDefaultHelp();
        }
      },
    },

    // Accessibility shortcuts
    {
      key: 'Escape',
      description: 'Close modals/overlays',
      action: () => {
        // Close any open modals or overlays
        const modals = document.querySelectorAll('[role="dialog"]');
        modals.forEach(modal => {
          const closeButton = modal.querySelector('[aria-label*="close"], [aria-label*="Close"]');
          if (closeButton instanceof HTMLElement) {
            closeButton.click();
          }
        });
      },
    },

    // Focus management
    {
      key: 'Tab',
      description: 'Navigate between focusable elements',
      action: () => {
        // Let browser handle default tab behavior
      },
    },
  ];

  shortcutsRef.current = shortcuts;

  const showDefaultHelp = useCallback(() => {
    if (helpModalRef.current) return;

    helpModalRef.current = true;

    // Create help modal
    const modal = document.createElement('div');
    modal.style.cssText = `
      position: fixed;
      top: 0;
      left: 0;
      right: 0;
      bottom: 0;
      background: rgba(0, 0, 0, 0.5);
      display: flex;
      align-items: center;
      justify-content: center;
      z-index: 10000;
      font-family: Inter, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    `;
    modal.setAttribute('role', 'dialog');
    modal.setAttribute('aria-labelledby', 'help-title');
    modal.setAttribute('aria-modal', 'true');

    const content = document.createElement('div');
    content.style.cssText = `
      background: var(--mantine-color-surface, white);
      color: var(--mantine-color-text, black);
      border-radius: 8px;
      padding: 24px;
      max-width: 600px;
      max-height: 80vh;
      overflow-y: auto;
      box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 10px 10px -5px rgba(0, 0, 0, 0.04);
    `;

    const title = document.createElement('h2');
    title.id = 'help-title';
    title.textContent = 'Keyboard Shortcuts';
    title.style.cssText = 'margin: 0 0 16px 0; font-size: 1.5rem; font-weight: 600;';

    const shortcutsList = document.createElement('div');
    shortcutsList.style.cssText = 'display: grid; gap: 8px;';

    shortcuts.forEach(shortcut => {
      const item = document.createElement('div');
      item.style.cssText = `
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 8px 0;
        border-bottom: 1px solid var(--mantine-color-border, #e0e0e0);
      `;

      const description = document.createElement('span');
      description.textContent = shortcut.description;
      description.style.cssText = 'flex: 1;';

      const keyCombo = document.createElement('code');
      const keys = [];
      if (shortcut.ctrlKey || shortcut.metaKey) keys.push(navigator.platform.includes('Mac') ? '⌘' : 'Ctrl');
      if (shortcut.altKey) keys.push(navigator.platform.includes('Mac') ? '⌥' : 'Alt');
      if (shortcut.shiftKey) keys.push('⇧');
      keys.push(shortcut.key === ' ' ? 'Space' : shortcut.key);

      keyCombo.textContent = keys.join(' + ');
      keyCombo.style.cssText = `
        background: var(--mantine-color-surface-secondary, #f5f5f5);
        padding: 4px 8px;
        border-radius: 4px;
        font-family: monospace;
        font-size: 0.875rem;
      `;

      item.appendChild(description);
      item.appendChild(keyCombo);
      shortcutsList.appendChild(item);
    });

    const closeButton = document.createElement('button');
    closeButton.textContent = 'Close';
    closeButton.style.cssText = `
      margin-top: 16px;
      padding: 8px 16px;
      background: var(--mantine-color-primary, #007bff);
      color: white;
      border: none;
      border-radius: 4px;
      cursor: pointer;
      font-size: 0.875rem;
    `;
    closeButton.setAttribute('aria-label', 'Close help dialog');

    const closeModal = () => {
      document.body.removeChild(modal);
      helpModalRef.current = false;
    };

    closeButton.addEventListener('click', closeModal);
    modal.addEventListener('click', (e) => {
      if (e.target === modal) closeModal();
    });

    content.appendChild(title);
    content.appendChild(shortcutsList);
    content.appendChild(closeButton);
    modal.appendChild(content);
    document.body.appendChild(modal);

    // Focus the close button
    closeButton.focus();
  }, [shortcuts]);

  const handleKeyDown = useCallback((event: KeyboardEvent) => {
    if (disabled) return;

    // Don't trigger shortcuts when typing in inputs
    const target = event.target as HTMLElement;
    if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable) {
      // Allow Escape to blur inputs
      if (event.key === 'Escape') {
        target.blur();
      }
      return;
    }

    const matchingShortcut = shortcuts.find(shortcut => {
      return (
        shortcut.key.toLowerCase() === event.key.toLowerCase() &&
        !!shortcut.ctrlKey === (event.ctrlKey || event.metaKey) &&
        !!shortcut.altKey === event.altKey &&
        !!shortcut.shiftKey === event.shiftKey
      );
    });

    if (matchingShortcut) {
      event.preventDefault();
      matchingShortcut.action(event);
    }
  }, [disabled, shortcuts]);

  useEffect(() => {
    if (disabled) return;

    document.addEventListener('keydown', handleKeyDown);
    return () => {
      document.removeEventListener('keydown', handleKeyDown);
    };
  }, [handleKeyDown, disabled]);

  // Return shortcuts for external use (e.g., help display)
  return {
    shortcuts: shortcutsRef.current,
    showHelp: showDefaultHelp,
  };
}

export default useKeyboardShortcuts;
