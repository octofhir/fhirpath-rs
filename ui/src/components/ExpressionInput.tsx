import { Component, createSignal, Show } from 'solid-js';
import styles from './ExpressionInput.module.css';

interface ExpressionInputProps {
  expression: string;
  onExpressionChange: (expression: string) => void;
  disabled?: boolean;
}

const ExpressionInput: Component<ExpressionInputProps> = (props) => {
  const [validationStatus, setValidationStatus] = createSignal<{
    valid: boolean;
    message: string;
  }>({ valid: true, message: '' });

  let textareaRef: HTMLTextAreaElement | undefined;

  const examples = [
    'Patient.name.given',
    'Patient.birthDate',
    "Patient.telecom.where(system = 'email')",
    'Bundle.entry.resource.where($this is Patient)',
    'Observation.value.as(Quantity).value',
    "Patient.name.select(given.join(' ') + ' ' + family)",
  ];

  const handleInputChange = (e: Event) => {
    const target = e.target as HTMLTextAreaElement;
    const value = target.value;
    props.onExpressionChange(value);
    
    // Basic validation - just check if it's not empty
    if (value.trim()) {
      setValidationStatus({ valid: true, message: 'Expression ready' });
    } else {
      setValidationStatus({ valid: true, message: 'Enter a FHIRPath expression' });
    }
  };

  const insertExample = (example: string) => {
    if (props.disabled) return;
    
    const textarea = textareaRef;
    if (!textarea) return;

    const start = textarea.selectionStart;
    const end = textarea.selectionEnd;
    const currentValue = props.expression;
    
    const newValue = currentValue.substring(0, start) + example + currentValue.substring(end);
    props.onExpressionChange(newValue);
    
    // Set cursor position after inserted text
    setTimeout(() => {
      textarea.focus();
      textarea.setSelectionRange(start + example.length, start + example.length);
    }, 0);
  };

  const handleClear = () => {
    if (props.disabled) return;
    props.onExpressionChange('');
    textareaRef?.focus();
  };


  const getStatusIcon = () => {
    if (!props.expression.trim()) return '📝';
    return validationStatus().valid ? '✅' : '❌';
  };

  const getStatusClass = () => {
    if (!props.expression.trim()) return styles.statusNeutral;
    return validationStatus().valid ? styles.statusValid : styles.statusInvalid;
  };

  return (
    <div class={styles.expressionInput}>
      <div class={styles.textareaContainer}>
        <textarea
          ref={textareaRef}
          class={styles.textarea}
          value={props.expression}
          onInput={handleInputChange}
          disabled={props.disabled}
          placeholder="Enter your FHIRPath expression here...

Examples:
• Patient.name.given
• Observation.value.as(Quantity)
• Bundle.entry.resource.where($this is Patient)"
          spellcheck={false}
          autocomplete="off"
        />
      </div>

      <div class={styles.characterCount}>
        <div class={`${styles.validationStatus} ${getStatusClass()}`}>
          <span class={styles.statusIcon}>{getStatusIcon()}</span>
          <span>{validationStatus().message}</span>
        </div>
        <span>{props.expression.length} characters</span>
      </div>

      <div class={styles.quickActions}>
        <button 
          class={styles.actionButton}
          onClick={handleClear}
          disabled={props.disabled || !props.expression.trim()}
          title="Clear expression"
        >
          🗑️ Clear
        </button>
      </div>

      <Show when={!props.disabled}>
        <div class={styles.examples}>
          <div class={styles.examplesTitle}>Quick Examples:</div>
          <div class={styles.examplesList}>
            {examples.map((example) => (
              <button
                class={styles.exampleButton}
                onClick={() => insertExample(example)}
                disabled={props.disabled}
                title={`Insert: ${example}`}
              >
                {example}
              </button>
            ))}
          </div>
        </div>
      </Show>
    </div>
  );
};

export default ExpressionInput;