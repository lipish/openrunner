import React from 'react';

const labelStyle = {
  display: 'block',
  fontSize: 12,
  fontWeight: 500,
  color: '#374151',
  marginBottom: 6,
};

const hintStyle = {
  fontSize: 11,
  color: '#8E8E93',
  marginTop: 6,
};

const warningStyle = {
  fontSize: 11,
  color: '#F59E0B',
  marginTop: 6,
};

export function Label({ children, style, ...props }) {
  return (
    <label style={{ ...labelStyle, ...style }} {...props}>
      {children}
    </label>
  );
}

export function Hint({ children, style, ...props }) {
  return (
    <div style={{ ...hintStyle, ...style }} {...props}>
      {children}
    </div>
  );
}

export function Warning({ children, style, ...props }) {
  return (
    <div style={{ ...warningStyle, ...style }} {...props}>
      ⚠️ {children}
    </div>
  );
}

