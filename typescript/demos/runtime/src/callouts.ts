
import { css } from 'lit';

export default css`
/* Borrowed from https://shoelace.style/getting-started/themes */
.callout {
  --docs-content-vertical-spacing: 2rem;
  --docs-border-radius: 0.25rem;

  position: relative;
  background-color: var(--sl-color-neutral-100);
  border-left: solid 4px var(--sl-color-neutral-500);
  border-radius: var(--docs-border-radius);
  color: var(--sl-color-neutral-800);
  padding: 1.5rem 1.5rem 1.5rem 2rem;
  margin-bottom: var(--docs-content-vertical-spacing);
}

.callout > :first-child {
  margin-top: 0;
}

.callout > :last-child {
  margin-bottom: 0;
}

.callout--tip {
  background-color: var(--sl-color-primary-100);
  border-left-color: var(--sl-color-primary-600);
  color: var(--sl-color-primary-800);
}

.callout::before {
  content: '';
  position: absolute;
  top: calc(50% - 0.8rem);
  left: calc(-0.8rem - 2px);
  width: 1.6rem;
  height: 1.6rem;
  display: flex;
  align-items: center;
  justify-content: center;
  font-family: var(--sl-font-serif);
  font-weight: var(--sl-font-weight-bold);
  color: var(--sl-color-neutral-0);
  clip-path: circle(50% at 50% 50%);
}

.callout--tip::before {
  content: 'i';
  font-style: italic;
  background-color: var(--sl-color-primary-600);
}

.callout--warning {
  background-color: var(--sl-color-warning-100);
  border-left-color: var(--sl-color-warning-600);
  color: var(--sl-color-warning-800);
}

.callout--warning::before {
  content: '!';
  background-color: var(--sl-color-warning-600);
}

.callout--danger {
  background-color: var(--sl-color-danger-100);
  border-left-color: var(--sl-color-danger-600);
  color: var(--sl-color-danger-800);
}

.callout--danger::before {
  content: '‼';
  background-color: var(--sl-color-danger-600);
}

.callout + .callout {
  margin-top: calc(-0.5 * var(--docs-content-vertical-spacing));
}

.callout a {
  color: inherit;
}
`;
