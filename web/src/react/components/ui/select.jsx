import React from 'react';

const selectStyle = {
  width: '100%',
  padding: '10px 12px',
  border: '1px solid #D1D5DB',
  borderRadius: 8,
  fontSize: 14,
  background: '#fff',
  cursor: 'pointer',
  outline: 'none',
  transition: 'border-color 0.15s, box-shadow 0.15s',
};

const selectFocusStyle = {
  borderColor: '#007AFF',
  boxShadow: '0 0 0 2px rgba(0, 122, 255, 0.15)',
};

export function Select({ value, onChange, children, className, style, ...props }) {
  const [focused, setFocused] = React.useState(false);

  return (
    <select
      value={value}
      onChange={onChange}
      onFocus={() => setFocused(true)}
      onBlur={() => setFocused(false)}
      style={{ ...selectStyle, ...(focused ? selectFocusStyle : {}), ...style }}
      {...props}
    >
      {children}
    </select>
  );
}

export function SelectOption({ value, children, ...props }) {
  return (
    <option value={value} {...props}>
      {children}
    </option>
  );
}

