import React from 'react';

const baseStyle = {
  display: 'inline-flex',
  alignItems: 'center',
  justifyContent: 'center',
  gap: 6,
  padding: '8px 16px',
  borderRadius: 8,
  fontSize: 14,
  fontWeight: 500,
  cursor: 'pointer',
  outline: 'none',
  transition: 'background-color 0.15s, opacity 0.15s, box-shadow 0.15s',
  border: 'none',
};

const variants = {
  default: {
    background: '#007AFF',
    color: '#fff',
  },
  secondary: {
    background: '#F2F2F7',
    color: '#1C1C1E',
    border: '1px solid #D1D5DB',
  },
  outline: {
    background: 'transparent',
    color: '#1C1C1E',
    border: '1px solid #D1D5DB',
  },
  ghost: {
    background: 'transparent',
    color: '#1C1C1E',
  },
  destructive: {
    background: '#FF3B30',
    color: '#fff',
  },
};

const sizes = {
  sm: {
    padding: '6px 12px',
    fontSize: 13,
  },
  md: {
    padding: '8px 16px',
    fontSize: 14,
  },
  lg: {
    padding: '10px 20px',
    fontSize: 15,
  },
};

export function Button({
  children,
  onClick,
  disabled,
  variant = 'default',
  size = 'md',
  type = 'button',
  style,
  ...props
}) {
  const variantStyle = variants[variant] || variants.default;
  const sizeStyle = sizes[size] || sizes.md;

  const disabledStyle = disabled
    ? {
        opacity: 0.5,
        cursor: 'not-allowed',
      }
    : {};

  return (
    <button
      type={type}
      onClick={disabled ? undefined : onClick}
      disabled={disabled}
      style={{ ...baseStyle, ...variantStyle, ...sizeStyle, ...disabledStyle, ...style }}
      {...props}
    >
      {children}
    </button>
  );
}

