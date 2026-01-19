import React, { createContext, useContext, useEffect } from 'react';
import { createPortal } from 'react-dom';

const DialogContext = createContext(null);

export function Dialog({ open, onOpenChange, children }) {
  return <DialogContext.Provider value={{ open, onOpenChange }}>{children}</DialogContext.Provider>;
}

export function DialogContent({ children }) {
  const ctx = useContext(DialogContext);
  const open = Boolean(ctx?.open);

  useEffect(() => {
    if (!open) return;
    const onKeyDown = (e) => {
      if (e.key === 'Escape') ctx?.onOpenChange?.(false);
    };
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, [open, ctx]);

  if (!open) return null;

  const overlayStyle = {
    position: 'fixed',
    inset: 0,
    background: 'rgba(0,0,0,0.35)',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    padding: 16,
    zIndex: 1000,
  };

  const contentStyle = {
    width: 'min(520px, 100%)',
    background: '#fff',
    borderRadius: 16,
    boxShadow: '0 12px 30px rgba(0,0,0,0.22)',
    overflow: 'hidden',
  };

  return createPortal(
    <div
      style={overlayStyle}
      onMouseDown={() => ctx?.onOpenChange?.(false)}
      role="dialog"
      aria-modal="true"
    >
      <div style={contentStyle} onMouseDown={(e) => e.stopPropagation()}>
        {children}
      </div>
    </div>,
    document.body
  );
}

export function DialogHeader({ children }) {
  return <div style={{ padding: '16px 20px', borderBottom: '1px solid #E5E5EA', background: '#FAFAFA' }}>{children}</div>;
}

export function DialogTitle({ children }) {
  return <div style={{ margin: 0, fontSize: 15, fontWeight: 600 }}>{children}</div>;
}

export function DialogBody({ children }) {
  return <div style={{ padding: 20, fontSize: 14, color: '#111' }}>{children}</div>;
}

export function DialogFooter({ children }) {
  return <div style={{ padding: 16, display: 'flex', justifyContent: 'flex-end', gap: 8, borderTop: '1px solid #E5E5EA' }}>{children}</div>;
}
