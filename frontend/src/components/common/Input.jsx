import './Input.css';

/**
 * Doc 4 §2.2: rounded (12px), dark surface bg, icon-prefixed, 48px height,
 * gold focus-ring on tap. Error: red text below field + red border on
 * that input only (§2.2). §4 Accessibility: errors announced inline via
 * aria-describedby, not only a top banner.
 */
export default function Input({
  icon,               // e.g. <UserIcon /> — person/envelope/lock icon
  label,
  type = 'text',
  value,
  onChange,
  placeholder,
  error,
  id,
  ...rest
}) {
  const errorId = error ? `${id}-error` : undefined;

  return (
    <div className="ck-input-group">
      {label && <label htmlFor={id} className="ck-input-label">{label}</label>}
      <div className={`ck-input-wrap ${error ? 'ck-input-wrap--error' : ''}`}>
        {icon && <span className="ck-input-icon" aria-hidden="true">{icon}</span>}
        <input
          id={id}
          type={type}
          value={value}
          onChange={onChange}
          placeholder={placeholder}
          aria-invalid={!!error}
          aria-describedby={errorId}
          className="ck-input"
          {...rest}
        />
      </div>
      {error && (
        <p id={errorId} className="ck-input-error" role="alert">
          {error}
        </p>
      )}
    </div>
  );
}
