import React from 'react';

const inputStyle = {
  width: '100%',
  padding: '10px 12px',
  border: '1px solid #D1D5DB',
  borderRadius: 8,
  fontSize: 14,
  background: '#fff',
  outline: 'none',
  transition: 'border-color 0.15s, box-shadow 0.15s',
};

const inputFocusStyle = {
  borderColor: '#007AFF',
  boxShadow: '0 0 0 2px rgba(0, 122, 255, 0.15)',
};

export function Input({ value, onChange, placeholder, type = 'text', className, style, ...props }) {
  const [focused, setFocused] = React.useState(false);

  return (
    <input
      type={type}
      value={value}
      onChange={onChange}
      placeholder={placeholder}
      onFocus={() => setFocused(true)}
      onBlur={() => setFocused(false)}
      style={{ ...inputStyle, ...(focused ? inputFocusStyle : {}), ...style }}
      {...props}
    />
  );
}

const textareaStyle = {
  width: '100%',
  padding: '10px 12px',
  border: '1px solid #D1D5DB',
  borderRadius: 8,
  fontSize: 13,
  fontFamily: 'ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace',
  background: '#fff',
  outline: 'none',
  resize: 'vertical',
  transition: 'border-color 0.15s, box-shadow 0.15s',
};

export function Textarea({ value, onChange, placeholder, rows = 4, className, style, ...props }) {
  const [focused, setFocused] = React.useState(false);

  return (
    <textarea
      value={value}
      onChange={onChange}
      placeholder={placeholder}
      rows={rows}
      onFocus={() => setFocused(true)}
      onBlur={() => setFocused(false)}
      style={{ ...textareaStyle, ...(focused ? inputFocusStyle : {}), ...style }}
      {...props}
    />
  );
}

