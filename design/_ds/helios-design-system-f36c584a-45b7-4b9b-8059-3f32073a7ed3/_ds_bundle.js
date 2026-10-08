/* @ds-bundle: {"format":3,"namespace":"HeliosDesignSystem_f36c58","components":[{"name":"Button","sourcePath":"components/actions/Button.jsx"},{"name":"IconButton","sourcePath":"components/actions/IconButton.jsx"},{"name":"Card","sourcePath":"components/containers/Card.jsx"},{"name":"Alert","sourcePath":"components/feedback/Alert.jsx"},{"name":"Badge","sourcePath":"components/feedback/Badge.jsx"},{"name":"Checkbox","sourcePath":"components/forms/Checkbox.jsx"},{"name":"Select","sourcePath":"components/forms/Select.jsx"},{"name":"TextInput","sourcePath":"components/forms/TextInput.jsx"},{"name":"Toggle","sourcePath":"components/forms/Toggle.jsx"},{"name":"Avatar","sourcePath":"components/media/Avatar.jsx"},{"name":"Icon","sourcePath":"components/media/Icon.jsx"},{"name":"Tabs","sourcePath":"components/navigation/Tabs.jsx"}],"sourceHashes":{"components/actions/Button.jsx":"2feac429e03e","components/actions/IconButton.jsx":"a05886faf5ee","components/containers/Card.jsx":"93353815c119","components/feedback/Alert.jsx":"5d6b17475215","components/feedback/Badge.jsx":"5e4ed0579c2f","components/forms/Checkbox.jsx":"5b3af5547961","components/forms/Select.jsx":"c809e15a659e","components/forms/TextInput.jsx":"299491e29a27","components/forms/Toggle.jsx":"624b7e3f25de","components/media/Avatar.jsx":"fd84ac1ddbf6","components/media/Icon.jsx":"7bfcaaf6145e","components/navigation/Tabs.jsx":"345e334072bf","ui_kits/hcp/AppHeader.jsx":"64d337d894c5","ui_kits/hcp/LoginScreen.jsx":"26843e3b1250","ui_kits/hcp/OverviewScreen.jsx":"4f12ab72e7bf","ui_kits/hcp/SideNav.jsx":"a2d6b2a43cab","ui_kits/hcp/WorkspaceDetail.jsx":"85141d0c4e92","ui_kits/hcp/WorkspacesScreen.jsx":"06d24fd32db5","ui_kits/hcp/data.js":"7b13d6432e53"},"inlinedExternals":[],"unexposedExports":[]} */

(() => {

const __ds_ns = (window.HeliosDesignSystem_f36c58 = window.HeliosDesignSystem_f36c58 || {});

const __ds_scope = {};

(__ds_ns.__errors = __ds_ns.__errors || []);

// components/actions/Button.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
const CSS = `
.hds-btn {
  display: inline-flex; align-items: center; justify-content: center;
  gap: 6px; box-sizing: border-box;
  font-family: var(--token-typography-font-stack-display);
  font-weight: var(--token-typography-font-weight-medium);
  border-radius: var(--token-border-radius-small);
  border: 1px solid transparent;
  cursor: pointer; white-space: nowrap; text-decoration: none;
  transition: background-color var(--hds-duration-fast) ease, box-shadow var(--hds-duration-fast) ease, color var(--hds-duration-fast) ease;
}
.hds-btn:focus-visible { outline: none; box-shadow: var(--token-focus-ring-action-box-shadow); }
.hds-btn--small  { height: 28px; padding: 0 10px; font-size: 0.8125rem; }
.hds-btn--medium { height: 36px; padding: 0 14px; font-size: 0.875rem; }
.hds-btn--large  { height: 44px; padding: 0 18px; font-size: 1rem; }
.hds-btn[disabled] { cursor: not-allowed; opacity: 1;
  background: var(--token-color-surface-interactive-disabled) !important;
  color: var(--token-color-foreground-disabled) !important;
  border-color: var(--token-color-border-faint) !important; box-shadow: none !important; }

.hds-btn--primary { background: var(--token-color-foreground-action); color: #fff; }
.hds-btn--primary:hover:not([disabled]) { background: var(--token-color-foreground-action-hover); }
.hds-btn--primary:active:not([disabled]) { background: var(--token-color-foreground-action-active); }

.hds-btn--secondary { background: var(--token-color-surface-interactive); color: var(--token-color-foreground-strong);
  box-shadow: var(--token-surface-base-box-shadow); border-color: transparent; }
.hds-btn--secondary:hover:not([disabled]) { background: var(--token-color-surface-interactive-hover); }
.hds-btn--secondary:active:not([disabled]) { background: var(--token-color-surface-interactive-active); }

.hds-btn--tertiary { background: transparent; color: var(--token-color-foreground-action); padding-left: 6px; padding-right: 6px; }
.hds-btn--tertiary:hover:not([disabled]) { background: var(--token-color-surface-action); }
.hds-btn--tertiary:active:not([disabled]) { background: var(--token-color-border-action); }

.hds-btn--critical { background: var(--token-color-foreground-critical-on-surface); color: #fff; }
.hds-btn--critical:hover:not([disabled]) { background: #940004; }
.hds-btn--critical:active:not([disabled]) { background: #51130a; }
.hds-btn--critical:focus-visible { box-shadow: var(--token-focus-ring-critical-box-shadow); }

.hds-btn__icon { width: 16px; height: 16px; display: inline-flex; flex: 0 0 auto; }
.hds-btn__icon svg, .hds-btn__icon img { width: 100%; height: 100%; display: block; }
`;
if (typeof document !== "undefined" && !document.getElementById("hds-button-css")) {
  const s = document.createElement("style");
  s.id = "hds-button-css";
  s.textContent = CSS;
  document.head.appendChild(s);
}
function Button({
  children,
  color = "primary",
  size = "medium",
  icon = null,
  iconPosition = "leading",
  isFullWidth = false,
  type = "button",
  href,
  style,
  ...rest
}) {
  const cls = `hds-btn hds-btn--${color} hds-btn--${size}`;
  const mergedStyle = isFullWidth ? {
    width: "100%",
    ...style
  } : style;
  const iconEl = icon ? /*#__PURE__*/React.createElement("span", {
    className: "hds-btn__icon"
  }, icon) : null;
  const content = /*#__PURE__*/React.createElement(React.Fragment, null, iconPosition === "leading" && iconEl, children != null && /*#__PURE__*/React.createElement("span", null, children), iconPosition === "trailing" && iconEl);
  if (href) {
    return /*#__PURE__*/React.createElement("a", _extends({
      className: cls,
      href: href,
      style: mergedStyle
    }, rest), content);
  }
  return /*#__PURE__*/React.createElement("button", _extends({
    className: cls,
    type: type,
    style: mergedStyle
  }, rest), content);
}
Object.assign(__ds_scope, { Button });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/actions/Button.jsx", error: String((e && e.message) || e) }); }

// components/actions/IconButton.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
const CSS = `
.hds-iconbtn {
  display: inline-flex; align-items: center; justify-content: center;
  box-sizing: border-box; cursor: pointer; padding: 0;
  border-radius: var(--token-border-radius-small);
  border: 1px solid transparent; background: transparent;
  color: var(--token-color-foreground-faint);
  transition: background-color var(--hds-duration-fast) ease, color var(--hds-duration-fast) ease, box-shadow var(--hds-duration-fast) ease;
}
.hds-iconbtn:focus-visible { outline: none; box-shadow: var(--token-focus-ring-action-box-shadow); }
.hds-iconbtn--small  { width: 28px; height: 28px; }
.hds-iconbtn--medium { width: 36px; height: 36px; }
.hds-iconbtn--large  { width: 44px; height: 44px; }
.hds-iconbtn__icon { width: 16px; height: 16px; display: block; }
.hds-iconbtn__icon svg, .hds-iconbtn__icon img { width: 100%; height: 100%; display: block; }
.hds-iconbtn[disabled] { cursor: not-allowed; color: var(--token-color-foreground-disabled) !important; background: transparent !important; }

.hds-iconbtn--ghost:hover:not([disabled]) { background: var(--token-color-surface-interactive-hover); color: var(--token-color-foreground-strong); }
.hds-iconbtn--ghost:active:not([disabled]) { background: var(--token-color-surface-interactive-active); }

.hds-iconbtn--secondary { background: var(--token-color-surface-interactive); color: var(--token-color-foreground-strong); box-shadow: var(--token-surface-base-box-shadow); }
.hds-iconbtn--secondary:hover:not([disabled]) { background: var(--token-color-surface-interactive-hover); }
.hds-iconbtn--secondary:active:not([disabled]) { background: var(--token-color-surface-interactive-active); }
`;
if (typeof document !== "undefined" && !document.getElementById("hds-iconbutton-css")) {
  const s = document.createElement("style");
  s.id = "hds-iconbutton-css";
  s.textContent = CSS;
  document.head.appendChild(s);
}
function IconButton({
  icon,
  color = "ghost",
  size = "medium",
  "aria-label": ariaLabel,
  style,
  ...rest
}) {
  return /*#__PURE__*/React.createElement("button", _extends({
    className: `hds-iconbtn hds-iconbtn--${color} hds-iconbtn--${size}`,
    "aria-label": ariaLabel,
    style: style
  }, rest), /*#__PURE__*/React.createElement("span", {
    className: "hds-iconbtn__icon"
  }, icon));
}
Object.assign(__ds_scope, { IconButton });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/actions/IconButton.jsx", error: String((e && e.message) || e) }); }

// components/containers/Card.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
const CSS = `
.hds-card {
  display: block; box-sizing: border-box;
  background: var(--token-color-surface-primary);
  border-radius: var(--token-border-radius-medium);
}
.hds-card--base   { box-shadow: var(--token-surface-base-box-shadow); }
.hds-card--low    { box-shadow: var(--token-surface-low-box-shadow); }
.hds-card--mid    { box-shadow: var(--token-surface-mid-box-shadow); }
.hds-card--high   { box-shadow: var(--token-surface-high-box-shadow); }
.hds-card--higher { box-shadow: var(--token-surface-higher-box-shadow); }
.hds-card--hidden { overflow: hidden; }
.hds-card--faint { background: var(--token-color-surface-faint); }
.hds-card--pad-small  { padding: 16px; }
.hds-card--pad-medium { padding: 24px; }
.hds-card--pad-large  { padding: 32px; }
.hds-card--interactive { cursor: pointer; transition: box-shadow var(--hds-duration-base) ease, transform var(--hds-duration-base) ease; }
.hds-card--interactive:hover { box-shadow: var(--token-surface-mid-box-shadow); }
`;
if (typeof document !== "undefined" && !document.getElementById("hds-card-css")) {
  const s = document.createElement("style");
  s.id = "hds-card-css";
  s.textContent = CSS;
  document.head.appendChild(s);
}
function Card({
  children,
  level = "base",
  padding = "medium",
  background = "primary",
  overflow = "visible",
  isInteractive = false,
  style,
  className = "",
  ...rest
}) {
  const cls = ["hds-card", `hds-card--${level}`, padding ? `hds-card--pad-${padding}` : "", background === "faint" ? "hds-card--faint" : "", overflow === "hidden" ? "hds-card--hidden" : "", isInteractive ? "hds-card--interactive" : "", className].filter(Boolean).join(" ");
  return /*#__PURE__*/React.createElement("div", _extends({
    className: cls,
    style: style
  }, rest), children);
}
Object.assign(__ds_scope, { Card });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/containers/Card.jsx", error: String((e && e.message) || e) }); }

// components/feedback/Alert.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
const CSS = `
.hds-alert {
  display: flex; gap: 12px; box-sizing: border-box; padding: 14px 16px;
  border-radius: var(--token-border-radius-medium); border: 1px solid transparent;
  font-family: var(--token-typography-font-stack-text); position: relative;
}
.hds-alert--neutral   { background: var(--token-color-surface-faint);     border-color: var(--token-color-border-primary); }
.hds-alert--highlight { background: var(--token-color-surface-highlight);  border-color: var(--token-color-border-highlight); }
.hds-alert--success   { background: var(--token-color-surface-success);    border-color: var(--token-color-border-success); }
.hds-alert--warning   { background: var(--token-color-surface-warning);    border-color: var(--token-color-border-warning); }
.hds-alert--critical  { background: var(--token-color-surface-critical);   border-color: var(--token-color-border-critical); }
.hds-alert__icon { width: 16px; height: 16px; flex: 0 0 auto; margin-top: 1px; }
.hds-alert__icon svg, .hds-alert__icon img { width: 100%; height: 100%; display: block; }
.hds-alert--neutral   .hds-alert__icon { color: var(--token-color-foreground-faint); }
.hds-alert--highlight .hds-alert__icon { color: var(--token-color-foreground-highlight-on-surface); }
.hds-alert--success   .hds-alert__icon { color: var(--token-color-foreground-success-on-surface); }
.hds-alert--warning   .hds-alert__icon { color: var(--token-color-foreground-warning-on-surface); }
.hds-alert--critical  .hds-alert__icon { color: var(--token-color-foreground-critical-on-surface); }
.hds-alert__body { display: flex; flex-direction: column; gap: 2px; flex: 1; min-width: 0; }
.hds-alert__title { font-size: 0.875rem; font-weight: var(--token-typography-font-weight-semibold); color: var(--token-color-foreground-strong); }
.hds-alert__desc { font-size: 0.875rem; line-height: 1.43; color: var(--token-color-foreground-primary); }
.hds-alert__actions { display: flex; gap: 12px; margin-top: 8px; }
.hds-alert__dismiss {
  flex: 0 0 auto; width: 20px; height: 20px; margin: -2px -4px 0 0; padding: 0; border: none; background: transparent;
  color: var(--token-color-foreground-faint); cursor: pointer; border-radius: var(--token-border-radius-x-small);
  display: inline-flex; align-items: center; justify-content: center;
}
.hds-alert__dismiss:hover { background: var(--token-color-surface-interactive-hover); color: var(--token-color-foreground-strong); }
`;
if (typeof document !== "undefined" && !document.getElementById("hds-alert-css")) {
  const s = document.createElement("style");
  s.id = "hds-alert-css";
  s.textContent = CSS;
  document.head.appendChild(s);
}
const DEFAULT_ICON = {
  neutral: /*#__PURE__*/React.createElement("svg", {
    viewBox: "0 0 16 16",
    fill: "currentColor",
    "aria-hidden": "true"
  }, /*#__PURE__*/React.createElement("path", {
    d: "M8 1a7 7 0 100 14A7 7 0 008 1zm.75 6.25a.75.75 0 00-1.5 0V11a.75.75 0 001.5 0V7.25zM8 4a1 1 0 100 2 1 1 0 000-2z"
  })),
  highlight: /*#__PURE__*/React.createElement("svg", {
    viewBox: "0 0 16 16",
    fill: "currentColor",
    "aria-hidden": "true"
  }, /*#__PURE__*/React.createElement("path", {
    d: "M8 1a7 7 0 100 14A7 7 0 008 1zm.75 6.25a.75.75 0 00-1.5 0V11a.75.75 0 001.5 0V7.25zM8 4a1 1 0 100 2 1 1 0 000-2z"
  })),
  success: /*#__PURE__*/React.createElement("svg", {
    viewBox: "0 0 16 16",
    fill: "currentColor",
    "aria-hidden": "true"
  }, /*#__PURE__*/React.createElement("path", {
    d: "M8 1a7 7 0 100 14A7 7 0 008 1zm3.03 5.28a.75.75 0 10-1.06-1.06L7 8.19 5.78 6.97a.75.75 0 00-1.06 1.06l1.75 1.75a.75.75 0 001.06 0l3.5-3.5z"
  })),
  warning: /*#__PURE__*/React.createElement("svg", {
    viewBox: "0 0 16 16",
    fill: "currentColor",
    "aria-hidden": "true"
  }, /*#__PURE__*/React.createElement("path", {
    d: "M8.93 1.6a1.06 1.06 0 00-1.86 0L.6 13.36A1 1 0 001.48 15h13.04a1 1 0 00.88-1.64L8.93 1.6zM8.75 6a.75.75 0 00-1.5 0v3.5a.75.75 0 001.5 0V6zM8 11a1 1 0 100 2 1 1 0 000-2z"
  })),
  critical: /*#__PURE__*/React.createElement("svg", {
    viewBox: "0 0 16 16",
    fill: "currentColor",
    "aria-hidden": "true"
  }, /*#__PURE__*/React.createElement("path", {
    d: "M8 1a7 7 0 100 14A7 7 0 008 1zm.75 4a.75.75 0 00-1.5 0v3.5a.75.75 0 001.5 0V5zM8 10.5a1 1 0 100 2 1 1 0 000-2z"
  }))
};
function Alert({
  color = "neutral",
  title,
  children,
  icon,
  actions,
  onDismiss,
  style,
  ...rest
}) {
  const iconNode = icon === null ? null : icon || DEFAULT_ICON[color];
  return /*#__PURE__*/React.createElement("div", _extends({
    className: `hds-alert hds-alert--${color}`,
    role: color === "critical" ? "alert" : "status",
    style: style
  }, rest), iconNode && /*#__PURE__*/React.createElement("span", {
    className: "hds-alert__icon"
  }, iconNode), /*#__PURE__*/React.createElement("div", {
    className: "hds-alert__body"
  }, title && /*#__PURE__*/React.createElement("div", {
    className: "hds-alert__title"
  }, title), children && /*#__PURE__*/React.createElement("div", {
    className: "hds-alert__desc"
  }, children), actions && /*#__PURE__*/React.createElement("div", {
    className: "hds-alert__actions"
  }, actions)), onDismiss && /*#__PURE__*/React.createElement("button", {
    className: "hds-alert__dismiss",
    "aria-label": "Dismiss",
    onClick: onDismiss
  }, /*#__PURE__*/React.createElement("svg", {
    width: "16",
    height: "16",
    viewBox: "0 0 16 16",
    fill: "currentColor",
    "aria-hidden": "true"
  }, /*#__PURE__*/React.createElement("path", {
    d: "M12.78 4.28a.75.75 0 00-1.06-1.06L8 6.94 4.28 3.22a.75.75 0 00-1.06 1.06L6.94 8l-3.72 3.72a.75.75 0 101.06 1.06L8 9.06l3.72 3.72a.75.75 0 101.06-1.06L9.06 8l3.72-3.72z"
  }))));
}
Object.assign(__ds_scope, { Alert });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/feedback/Alert.jsx", error: String((e && e.message) || e) }); }

// components/feedback/Badge.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
const CSS = `
.hds-badge {
  display: inline-flex; align-items: center; gap: 4px; box-sizing: border-box;
  font-family: var(--token-typography-font-stack-display); font-weight: var(--token-typography-font-weight-medium);
  border-radius: var(--token-border-radius-x-small); white-space: nowrap; border: 1px solid transparent;
}
.hds-badge--small  { height: 20px; padding: 0 6px; font-size: 0.6875rem; }
.hds-badge--medium { height: 24px; padding: 0 8px; font-size: 0.8125rem; }
.hds-badge--large  { height: 28px; padding: 0 10px; font-size: 0.875rem; }
.hds-badge__icon { width: 14px; height: 14px; display: inline-flex; }
.hds-badge__icon svg, .hds-badge__icon img { width: 100%; height: 100%; }

/* filled */
.hds-badge--filled.hds-badge--neutral   { background: var(--token-color-surface-strong); color: var(--token-color-foreground-primary); }
.hds-badge--filled.hds-badge--highlight { background: var(--token-color-surface-highlight); color: var(--token-color-foreground-highlight-on-surface); }
.hds-badge--filled.hds-badge--success   { background: var(--token-color-surface-success); color: var(--token-color-foreground-success-on-surface); }
.hds-badge--filled.hds-badge--warning   { background: var(--token-color-surface-warning); color: var(--token-color-foreground-warning-on-surface); }
.hds-badge--filled.hds-badge--critical  { background: var(--token-color-surface-critical); color: var(--token-color-foreground-critical-on-surface); }

/* outlined */
.hds-badge--outlined { background: var(--token-color-surface-primary); }
.hds-badge--outlined.hds-badge--neutral   { border-color: var(--token-color-border-strong); color: var(--token-color-foreground-primary); }
.hds-badge--outlined.hds-badge--highlight { border-color: var(--token-color-border-highlight); color: var(--token-color-foreground-highlight-on-surface); }
.hds-badge--outlined.hds-badge--success   { border-color: var(--token-color-border-success); color: var(--token-color-foreground-success-on-surface); }
.hds-badge--outlined.hds-badge--warning   { border-color: var(--token-color-border-warning); color: var(--token-color-foreground-warning-on-surface); }
.hds-badge--outlined.hds-badge--critical  { border-color: var(--token-color-border-critical); color: var(--token-color-foreground-critical-on-surface); }
`;
if (typeof document !== "undefined" && !document.getElementById("hds-badge-css")) {
  const s = document.createElement("style");
  s.id = "hds-badge-css";
  s.textContent = CSS;
  document.head.appendChild(s);
}
function Badge({
  text,
  children,
  color = "neutral",
  type = "filled",
  size = "medium",
  icon = null,
  style,
  ...rest
}) {
  return /*#__PURE__*/React.createElement("span", _extends({
    className: `hds-badge hds-badge--${type} hds-badge--${color} hds-badge--${size}`,
    style: style
  }, rest), icon && /*#__PURE__*/React.createElement("span", {
    className: "hds-badge__icon"
  }, icon), text ?? children);
}
Object.assign(__ds_scope, { Badge });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/feedback/Badge.jsx", error: String((e && e.message) || e) }); }

// components/forms/Checkbox.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
const CSS = `
.hds-choice { display: inline-flex; align-items: flex-start; gap: 8px; cursor: pointer; font-family: var(--token-typography-font-stack-text); }
.hds-choice input { position: absolute; opacity: 0; width: 0; height: 0; }
.hds-choice__box {
  box-sizing: border-box; width: 16px; height: 16px; flex: 0 0 auto; margin-top: 2px;
  background: var(--token-color-surface-primary);
  border: 1px solid var(--token-color-foreground-disabled);
  background-repeat: no-repeat; background-position: center; background-size: 12px;
  transition: background-color var(--hds-duration-fast) ease, border-color var(--hds-duration-fast) ease;
}
.hds-choice__box--checkbox { border-radius: var(--token-form-checkbox-border-radius, 3px); }
.hds-choice__box--radio { border-radius: 50%; }
.hds-choice:hover input:not(:disabled) ~ .hds-choice__box { border-color: var(--token-color-foreground-faint); }
.hds-choice input:checked ~ .hds-choice__box--checkbox {
  background-color: var(--token-color-foreground-action); border-color: var(--token-color-focus-action-internal);
  background-image: url("data:image/svg+xml,%3csvg viewBox='0 0 12 12' xmlns='http://www.w3.org/2000/svg'%3e%3cpath d='M9.78033 3.21967C10.0732 3.51256 10.0732 3.98744 9.78033 4.28033L5.28033 8.78033C4.98744 9.07322 4.51256 9.07322 4.21967 8.78033L2.21967 6.78033C1.92678 6.48744 1.92678 6.01256 2.21967 5.71967C2.51256 5.42678 2.98744 5.42678 3.28033 5.71967L4.75 7.18934L8.71967 3.21967C9.01256 2.92678 9.48744 2.92678 9.78033 3.21967Z' fill='%23FFF'/%3e%3c/svg%3e");
}
.hds-choice input:checked ~ .hds-choice__box--radio {
  background-color: var(--token-color-foreground-action); border-color: var(--token-color-focus-action-internal);
  background-image: url("data:image/svg+xml,%3csvg width='12' height='12' xmlns='http://www.w3.org/2000/svg'%3e%3ccircle cx='6' cy='6' r='2.5' fill='%23ffffff'/%3e%3c/svg%3e");
}
.hds-choice input:focus-visible ~ .hds-choice__box { box-shadow: var(--token-focus-ring-action-box-shadow); }
.hds-choice input:disabled ~ .hds-choice__box { background-color: var(--token-color-surface-interactive-disabled); border-color: var(--token-color-border-primary); }
.hds-choice input:disabled ~ .hds-choice__text { color: var(--token-color-foreground-disabled); }
.hds-choice__text { display: flex; flex-direction: column; gap: 2px; }
.hds-choice__label { font-size: 0.875rem; color: var(--token-color-foreground-strong); line-height: 1.4; }
.hds-choice__helper { font-size: 0.8125rem; color: var(--token-color-foreground-faint); }
`;
if (typeof document !== "undefined" && !document.getElementById("hds-choice-css")) {
  const s = document.createElement("style");
  s.id = "hds-choice-css";
  s.textContent = CSS;
  document.head.appendChild(s);
}
function Checkbox({
  label,
  helperText,
  type = "checkbox",
  style,
  ...rest
}) {
  const variant = type === "radio" ? "radio" : "checkbox";
  return /*#__PURE__*/React.createElement("label", {
    className: "hds-choice",
    style: style
  }, /*#__PURE__*/React.createElement("input", _extends({
    type: type
  }, rest)), /*#__PURE__*/React.createElement("span", {
    className: `hds-choice__box hds-choice__box--${variant}`,
    "aria-hidden": "true"
  }), (label || helperText) && /*#__PURE__*/React.createElement("span", {
    className: "hds-choice__text"
  }, label && /*#__PURE__*/React.createElement("span", {
    className: "hds-choice__label"
  }, label), helperText && /*#__PURE__*/React.createElement("span", {
    className: "hds-choice__helper"
  }, helperText)));
}
Object.assign(__ds_scope, { Checkbox });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/forms/Checkbox.jsx", error: String((e && e.message) || e) }); }

// components/forms/Select.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
const CSS = `
.hds-select {
  box-sizing: border-box; width: 100%; height: 36px; padding: 7px 30px 7px 11px;
  font-family: var(--token-typography-font-stack-text); font-size: 0.875rem;
  color: var(--token-color-foreground-strong);
  background-color: var(--token-color-surface-primary);
  background-image: url("data:image/svg+xml,%3Csvg viewBox='0 0 16 16' xmlns='http://www.w3.org/2000/svg'%3E%3Cpath d='M8.55 2.24a.75.75 0 0 0-1.1 0L4.2 5.74a.75.75 0 1 0 1.1 1.02L8 3.852l2.7 2.908a.75.75 0 1 0 1.1-1.02l-3.25-3.5Zm-1.1 11.52a.75.75 0 0 0 1.1 0l3.25-3.5a.75.75 0 1 0-1.1-1.02L8 12.148 5.3 9.24a.75.75 0 0 0-1.1 1.02l3.25 3.5Z' fill='%23656A76'/%3E%3C/svg%3E");
  background-repeat: no-repeat; background-position: right 7px center; background-size: 16px;
  border: 1px solid var(--token-color-foreground-disabled);
  border-radius: var(--token-border-radius-small);
  appearance: none; -webkit-appearance: none; cursor: pointer;
  transition: border-color var(--hds-duration-fast) ease, box-shadow var(--hds-duration-fast) ease;
}
.hds-select:hover { border-color: var(--token-color-foreground-faint); }
.hds-select:focus { outline: none; box-shadow: var(--token-focus-ring-action-box-shadow); border-color: var(--token-color-focus-action-internal); }
.hds-select:disabled { background-color: var(--token-color-surface-interactive-disabled); color: var(--token-color-foreground-disabled); cursor: not-allowed; }
`;
if (typeof document !== "undefined" && !document.getElementById("hds-select-css")) {
  const s = document.createElement("style");
  s.id = "hds-select-css";
  s.textContent = CSS;
  document.head.appendChild(s);
}
function Select({
  label,
  helperText,
  options = [],
  children,
  id,
  style,
  ...rest
}) {
  const selectId = id || (label ? "hds-select-" + label.replace(/\s+/g, "-").toLowerCase() : undefined);
  return /*#__PURE__*/React.createElement("div", {
    className: "hds-field",
    style: style
  }, label && /*#__PURE__*/React.createElement("label", {
    className: "hds-field__label",
    htmlFor: selectId
  }, label), helperText && /*#__PURE__*/React.createElement("span", {
    className: "hds-field__helper"
  }, helperText), /*#__PURE__*/React.createElement("select", _extends({
    id: selectId,
    className: "hds-select"
  }, rest), children || options.map(o => {
    const opt = typeof o === "string" ? {
      value: o,
      label: o
    } : o;
    return /*#__PURE__*/React.createElement("option", {
      key: opt.value,
      value: opt.value
    }, opt.label);
  })));
}
Object.assign(__ds_scope, { Select });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/forms/Select.jsx", error: String((e && e.message) || e) }); }

// components/forms/TextInput.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
const CSS = `
.hds-field { display: flex; flex-direction: column; gap: 4px; font-family: var(--token-typography-font-stack-text); }
.hds-field__label { font-size: 0.875rem; font-weight: var(--token-typography-font-weight-semibold); color: var(--token-color-foreground-strong); }
.hds-field__optional { font-weight: 400; color: var(--token-color-foreground-faint); }
.hds-field__helper { font-size: 0.8125rem; color: var(--token-color-foreground-faint); }
.hds-field__error { font-size: 0.8125rem; color: var(--token-color-foreground-critical-on-surface); display: flex; align-items: center; gap: 6px; }
.hds-input {
  box-sizing: border-box; width: 100%; height: 36px; padding: 7px 11px;
  font-family: inherit; font-size: 0.875rem; color: var(--token-color-foreground-strong);
  background: var(--token-color-surface-primary);
  border: 1px solid var(--token-color-foreground-disabled);
  border-radius: var(--token-border-radius-small);
  transition: border-color var(--hds-duration-fast) ease, box-shadow var(--hds-duration-fast) ease;
}
.hds-input::placeholder { color: var(--token-color-foreground-faint); }
.hds-input:hover { border-color: var(--token-color-foreground-faint); }
.hds-input:focus { outline: none; box-shadow: var(--token-focus-ring-action-box-shadow); border-color: var(--token-color-focus-action-internal); }
.hds-input--invalid { border-color: var(--token-color-foreground-critical-on-surface); }
.hds-input--invalid:focus { box-shadow: var(--token-focus-ring-critical-box-shadow); }
.hds-input:disabled { background: var(--token-color-surface-interactive-disabled); color: var(--token-color-foreground-disabled); border-color: var(--token-color-border-primary); cursor: not-allowed; }
.hds-input--has-icon { padding-left: 32px; }
.hds-input-wrap { position: relative; display: flex; align-items: center; }
.hds-input-wrap__icon { position: absolute; left: 9px; width: 16px; height: 16px; color: var(--token-color-foreground-faint); pointer-events: none; display: flex; }
.hds-input-wrap__icon svg, .hds-input-wrap__icon img { width: 100%; height: 100%; }
`;
if (typeof document !== "undefined" && !document.getElementById("hds-textinput-css")) {
  const s = document.createElement("style");
  s.id = "hds-textinput-css";
  s.textContent = CSS;
  document.head.appendChild(s);
}
function TextInput({
  label,
  helperText,
  errorText,
  isOptional = false,
  isInvalid = false,
  icon = null,
  id,
  style,
  ...rest
}) {
  const inputId = id || (label ? "hds-input-" + label.replace(/\s+/g, "-").toLowerCase() : undefined);
  const invalid = isInvalid || !!errorText;
  const inputEl = /*#__PURE__*/React.createElement("input", _extends({
    id: inputId,
    className: `hds-input ${invalid ? "hds-input--invalid" : ""} ${icon ? "hds-input--has-icon" : ""}`,
    "aria-invalid": invalid || undefined
  }, rest));
  return /*#__PURE__*/React.createElement("div", {
    className: "hds-field",
    style: style
  }, label && /*#__PURE__*/React.createElement("label", {
    className: "hds-field__label",
    htmlFor: inputId
  }, label, isOptional && /*#__PURE__*/React.createElement("span", {
    className: "hds-field__optional"
  }, " (optional)")), helperText && /*#__PURE__*/React.createElement("span", {
    className: "hds-field__helper"
  }, helperText), icon ? /*#__PURE__*/React.createElement("div", {
    className: "hds-input-wrap"
  }, /*#__PURE__*/React.createElement("span", {
    className: "hds-input-wrap__icon"
  }, icon), inputEl) : inputEl, errorText && /*#__PURE__*/React.createElement("span", {
    className: "hds-field__error"
  }, /*#__PURE__*/React.createElement("svg", {
    width: "14",
    height: "14",
    viewBox: "0 0 16 16",
    fill: "currentColor",
    "aria-hidden": "true"
  }, /*#__PURE__*/React.createElement("path", {
    d: "M8 1a7 7 0 100 14A7 7 0 008 1zm.75 4a.75.75 0 00-1.5 0v3.5a.75.75 0 001.5 0V5zM8 10.5a1 1 0 100 2 1 1 0 000-2z"
  })), errorText));
}
Object.assign(__ds_scope, { TextInput });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/forms/TextInput.jsx", error: String((e && e.message) || e) }); }

// components/forms/Toggle.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
const CSS = `
.hds-toggle { display: inline-flex; align-items: flex-start; gap: 8px; cursor: pointer; font-family: var(--token-typography-font-stack-text); }
.hds-toggle input { position: absolute; opacity: 0; width: 0; height: 0; }
.hds-toggle__track {
  box-sizing: border-box; position: relative; width: 32px; height: 16px; flex: 0 0 auto; margin-top: 1px;
  background: var(--token-color-surface-strong);
  border: 1px solid var(--token-color-foreground-disabled);
  border-radius: var(--token-border-radius-x-small);
  transition: background-color var(--hds-duration-base) var(--hds-ease-toggle), border-color var(--hds-duration-base) ease;
}
.hds-toggle__thumb {
  position: absolute; top: -1px; left: -1px; width: 16px; height: 16px;
  background: var(--token-color-surface-primary);
  border: 1px solid var(--token-color-foreground-disabled);
  border-radius: var(--token-border-radius-x-small);
  box-shadow: var(--token-elevation-low-box-shadow);
  transition: transform var(--hds-duration-base) var(--hds-ease-toggle), border-color var(--hds-duration-base) ease;
}
.hds-toggle:hover input:not(:disabled) ~ .hds-toggle__track { border-color: var(--token-color-foreground-faint); }
.hds-toggle input:checked ~ .hds-toggle__track { background: var(--token-color-foreground-action); border-color: var(--token-color-focus-action-internal); }
.hds-toggle input:checked ~ .hds-toggle__track .hds-toggle__thumb {
  transform: translateX(16px); border-color: var(--token-color-focus-action-internal);
  background-image: url("data:image/svg+xml,%3csvg viewBox='0 0 12 12' xmlns='http://www.w3.org/2000/svg'%3e%3cpath d='M9.78033 3.21967C10.0732 3.51256 10.0732 3.98744 9.78033 4.28033L5.28033 8.78033C4.98744 9.07322 4.51256 9.07322 4.21967 8.78033L2.21967 6.78033C1.92678 6.48744 1.92678 6.01256 2.21967 5.71967C2.51256 5.42678 2.98744 5.42678 3.28033 5.71967L4.75 7.18934L8.71967 3.21967C9.01256 2.92678 9.48744 2.92678 9.78033 3.21967Z' fill='%234f46e5'/%3e%3c/svg%3e");
  background-repeat: no-repeat; background-position: center; background-size: 10px;
}
.hds-toggle input:focus-visible ~ .hds-toggle__track { box-shadow: var(--token-focus-ring-action-box-shadow); }
.hds-toggle input:disabled ~ .hds-toggle__track { background: var(--token-color-surface-interactive-disabled); border-color: var(--token-color-border-primary); }
.hds-toggle__text { display: flex; flex-direction: column; gap: 2px; }
.hds-toggle__label { font-size: 0.875rem; color: var(--token-color-foreground-strong); line-height: 1.4; }
.hds-toggle__helper { font-size: 0.8125rem; color: var(--token-color-foreground-faint); }
`;
if (typeof document !== "undefined" && !document.getElementById("hds-toggle-css")) {
  const s = document.createElement("style");
  s.id = "hds-toggle-css";
  s.textContent = CSS;
  document.head.appendChild(s);
}
function Toggle({
  label,
  helperText,
  style,
  ...rest
}) {
  return /*#__PURE__*/React.createElement("label", {
    className: "hds-toggle",
    style: style
  }, /*#__PURE__*/React.createElement("input", _extends({
    type: "checkbox",
    role: "switch"
  }, rest)), /*#__PURE__*/React.createElement("span", {
    className: "hds-toggle__track",
    "aria-hidden": "true"
  }, /*#__PURE__*/React.createElement("span", {
    className: "hds-toggle__thumb"
  })), (label || helperText) && /*#__PURE__*/React.createElement("span", {
    className: "hds-toggle__text"
  }, label && /*#__PURE__*/React.createElement("span", {
    className: "hds-toggle__label"
  }, label), helperText && /*#__PURE__*/React.createElement("span", {
    className: "hds-toggle__helper"
  }, helperText)));
}
Object.assign(__ds_scope, { Toggle });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/forms/Toggle.jsx", error: String((e && e.message) || e) }); }

// components/media/Avatar.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
const CSS = `
.hds-avatar {
  display: inline-flex; align-items: center; justify-content: center; flex: 0 0 auto;
  font-family: var(--token-typography-font-stack-display); font-weight: var(--token-typography-font-weight-medium);
  color: #fff; overflow: hidden; background-size: cover; background-position: center; user-select: none;
}
.hds-avatar--small  { width: 24px; height: 24px; font-size: 10px; }
.hds-avatar--medium { width: 32px; height: 32px; font-size: 13px; }
.hds-avatar--large  { width: 40px; height: 40px; font-size: 16px; }
.hds-avatar--circle { border-radius: 50%; }
.hds-avatar--square { border-radius: var(--token-border-radius-small); }
`;
if (typeof document !== "undefined" && !document.getElementById("hds-avatar-css")) {
  const s = document.createElement("style");
  s.id = "hds-avatar-css";
  s.textContent = CSS;
  document.head.appendChild(s);
}
const PALETTE = ["var(--token-color-palette-blue-300)", "var(--token-color-palette-purple-300)", "var(--token-color-palette-green-300)", "var(--token-color-palette-amber-300)", "var(--token-color-palette-red-300)", "var(--token-color-palette-neutral-500)"];
function initials(name = "") {
  const parts = name.trim().split(/\s+/);
  if (parts.length === 0 || parts[0] === "") return "?";
  if (parts.length === 1) return parts[0].slice(0, 2).toUpperCase();
  return (parts[0][0] + parts[parts.length - 1][0]).toUpperCase();
}
function Avatar({
  name = "",
  src,
  size = "medium",
  shape = "circle",
  color,
  style,
  ...rest
}) {
  const bg = src ? undefined : color || PALETTE[name.split("").reduce((a, c) => a + c.charCodeAt(0), 0) % PALETTE.length];
  return /*#__PURE__*/React.createElement("span", _extends({
    className: `hds-avatar hds-avatar--${size} hds-avatar--${shape}`,
    style: {
      background: src ? `url(${src})` : bg,
      ...style
    },
    title: name || undefined
  }, rest), !src && initials(name));
}
Object.assign(__ds_scope, { Avatar });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/media/Avatar.jsx", error: String((e && e.message) || e) }); }

// components/media/Icon.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
/**
 * Renders a monochrome (Flight) icon by fetching the SVG and inlining it, so
 * the path's `fill="currentColor"` inherits the current text color — reliable
 * on colored buttons and in dark mode (an <img> ignores currentColor, and an
 * external-SVG CSS mask can silently fail depending on how the file is served).
 */
function Icon({
  src,
  size = 16,
  color,
  label,
  style,
  ...rest
}) {
  const [markup, setMarkup] = React.useState(() => Icon._cache[src] || null);
  React.useEffect(() => {
    let alive = true;
    if (Icon._cache[src]) {
      setMarkup(Icon._cache[src]);
      return;
    }
    fetch(src).then(r => r.text()).then(text => {
      if (!text || text.indexOf("<svg") === -1) return;
      // Let CSS control the box; keep the artwork crisp and centered.
      const normalized = text.replace(/<svg([^>]*)>/, '<svg$1 style="width:100%;height:100%;display:block">');
      Icon._cache[src] = normalized;
      if (alive) setMarkup(normalized);
    }).catch(() => {});
    return () => {
      alive = false;
    };
  }, [src]);
  return /*#__PURE__*/React.createElement("span", _extends({
    role: label ? "img" : "presentation",
    "aria-label": label || undefined,
    "aria-hidden": label ? undefined : true,
    style: {
      display: "inline-flex",
      width: size,
      height: size,
      flex: "0 0 auto",
      color: color || "currentColor",
      ...style
    },
    dangerouslySetInnerHTML: markup ? {
      __html: markup
    } : undefined
  }, rest));
}
Icon._cache = {};
Object.assign(__ds_scope, { Icon });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/media/Icon.jsx", error: String((e && e.message) || e) }); }

// components/navigation/Tabs.jsx
try { (() => {
const CSS = `
.hds-tabs { display: flex; flex-direction: column; font-family: var(--token-typography-font-stack-display); }
.hds-tabs__list { position: relative; display: flex; gap: 6px; border-bottom: 1px solid var(--token-color-border-primary); }
.hds-tabs__tab {
  position: relative; display: inline-flex; align-items: center; gap: 8px;
  background: transparent; border: none; cursor: pointer;
  color: var(--token-color-foreground-faint); font-weight: var(--token-typography-font-weight-medium);
  font-family: inherit; border-radius: var(--token-border-radius-small) var(--token-border-radius-small) 0 0;
  transition: color var(--hds-duration-fast) ease, background-color var(--hds-duration-fast) ease;
}
.hds-tabs__tab--medium { height: 36px; padding: 0 12px; font-size: 0.875rem; }
.hds-tabs__tab--large  { height: 48px; padding: 0 20px; font-size: 1rem; }
.hds-tabs__tab:hover { color: var(--token-color-foreground-strong); background: var(--token-color-surface-interactive-hover); }
.hds-tabs__tab--active { color: var(--token-color-foreground-strong); }
.hds-tabs__tab:focus-visible { outline: none; box-shadow: var(--token-focus-ring-action-box-shadow); }
.hds-tabs__icon { width: 16px; height: 16px; display: inline-flex; }
.hds-tabs__icon svg, .hds-tabs__icon img { width: 100%; height: 100%; }
.hds-tabs__count {
  display: inline-flex; align-items: center; justify-content: center; min-width: 20px; height: 18px; padding: 0 6px;
  font-size: 0.75rem; border-radius: 9px; background: var(--token-color-surface-strong); color: var(--token-color-foreground-faint);
}
.hds-tabs__tab--active .hds-tabs__count { background: var(--token-color-surface-action); color: var(--token-color-foreground-action); }
.hds-tabs__indicator {
  position: absolute; bottom: -1px; height: 3px; border-radius: 3px 3px 0 0;
  background: var(--token-color-foreground-action);
  transition: left var(--hds-tabs-indicator-transition-duration, 0.4s) var(--hds-ease-tabs), width var(--hds-tabs-indicator-transition-duration, 0.4s) var(--hds-ease-tabs);
}
`;
if (typeof document !== "undefined" && !document.getElementById("hds-tabs-css")) {
  const s = document.createElement("style");
  s.id = "hds-tabs-css";
  s.textContent = CSS;
  document.head.appendChild(s);
}
function Tabs({
  tabs = [],
  value,
  defaultValue,
  onChange,
  size = "medium",
  style
}) {
  const isControlled = value !== undefined;
  const [internal, setInternal] = React.useState(defaultValue ?? (tabs[0] && tabs[0].id));
  const active = isControlled ? value : internal;
  const listRef = React.useRef(null);
  const [indicator, setIndicator] = React.useState({
    left: 0,
    width: 0
  });
  React.useLayoutEffect(() => {
    const list = listRef.current;
    if (!list) return;
    const el = list.querySelector('[data-active="true"]');
    if (el) setIndicator({
      left: el.offsetLeft,
      width: el.offsetWidth
    });
  }, [active, tabs, size]);
  const select = id => {
    if (!isControlled) setInternal(id);
    onChange && onChange(id);
  };
  return /*#__PURE__*/React.createElement("div", {
    className: "hds-tabs",
    style: style
  }, /*#__PURE__*/React.createElement("div", {
    className: "hds-tabs__list",
    ref: listRef,
    role: "tablist"
  }, tabs.map(t => {
    const isActive = t.id === active;
    return /*#__PURE__*/React.createElement("button", {
      key: t.id,
      role: "tab",
      "aria-selected": isActive,
      "data-active": isActive,
      className: `hds-tabs__tab hds-tabs__tab--${size} ${isActive ? "hds-tabs__tab--active" : ""}`,
      onClick: () => select(t.id)
    }, t.icon && /*#__PURE__*/React.createElement("span", {
      className: "hds-tabs__icon"
    }, t.icon), t.label, t.count != null && /*#__PURE__*/React.createElement("span", {
      className: "hds-tabs__count"
    }, t.count));
  }), /*#__PURE__*/React.createElement("span", {
    className: "hds-tabs__indicator",
    style: {
      left: indicator.left,
      width: indicator.width
    }
  })));
}
Object.assign(__ds_scope, { Tabs });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/navigation/Tabs.jsx", error: String((e && e.message) || e) }); }

// ui_kits/hcp/AppHeader.jsx
try { (() => {
/* global React */
const HDR_ICN = "../../assets/icons/";
const HDR_BRAND = "../../assets/brand/";
function WindowBar({
  theme,
  onToggleTheme,
  user = "Mitchell Hashimoto",
  onSignOut
}) {
  const {
    Icon,
    IconButton,
    Avatar
  } = window.HeliosDesignSystem_f36c58;
  return /*#__PURE__*/React.createElement("header", {
    className: "kit-winbar"
  }, /*#__PURE__*/React.createElement("div", {
    className: "kit-lights"
  }, /*#__PURE__*/React.createElement("i", {
    style: {
      background: "#ff5f57"
    }
  }), /*#__PURE__*/React.createElement("i", {
    style: {
      background: "#febc2e"
    }
  }), /*#__PURE__*/React.createElement("i", {
    style: {
      background: "#28c840"
    }
  })), /*#__PURE__*/React.createElement("div", {
    className: "kit-winbar__title"
  }, /*#__PURE__*/React.createElement("img", {
    src: HDR_BRAND + "hashicorp-color-16.svg",
    alt: ""
  }), " HCP Terraform"), /*#__PURE__*/React.createElement("div", {
    className: "kit-winbar__right"
  }, /*#__PURE__*/React.createElement("div", {
    className: "kit-search"
  }, /*#__PURE__*/React.createElement(Icon, {
    src: HDR_ICN + "search-16.svg",
    size: 16
  }), /*#__PURE__*/React.createElement("input", {
    placeholder: "Search resources\u2026"
  }), /*#__PURE__*/React.createElement("kbd", null, "/")), /*#__PURE__*/React.createElement("div", {
    className: "kit-themeseg",
    role: "group",
    "aria-label": "Theme"
  }, /*#__PURE__*/React.createElement("button", {
    "aria-pressed": theme === "light",
    onClick: () => theme !== "light" && onToggleTheme()
  }, "Light"), /*#__PURE__*/React.createElement("button", {
    "aria-pressed": theme === "dark",
    onClick: () => theme !== "dark" && onToggleTheme()
  }, "Dark")), /*#__PURE__*/React.createElement(IconButton, {
    color: "ghost",
    "aria-label": "Help",
    icon: /*#__PURE__*/React.createElement(Icon, {
      src: HDR_ICN + "help-16.svg"
    })
  }), /*#__PURE__*/React.createElement(IconButton, {
    color: "ghost",
    "aria-label": "Notifications",
    icon: /*#__PURE__*/React.createElement(Icon, {
      src: HDR_ICN + "bell-16.svg"
    })
  }), /*#__PURE__*/React.createElement("button", {
    className: "kit-avatarbtn",
    onClick: onSignOut,
    title: "Sign out"
  }, /*#__PURE__*/React.createElement(Avatar, {
    name: user,
    size: "small"
  }))));
}
window.WindowBar = WindowBar;
window.AppHeader = WindowBar;
})(); } catch (e) { __ds_ns.__errors.push({ path: "ui_kits/hcp/AppHeader.jsx", error: String((e && e.message) || e) }); }

// ui_kits/hcp/LoginScreen.jsx
try { (() => {
/* global React */
const LOGIN_BRAND = "../../assets/brand/";
const LOGIN_ICN = "../../assets/icons/";
function LoginScreen({
  onLogin
}) {
  const {
    Button,
    TextInput,
    Icon,
    Checkbox
  } = window.HeliosDesignSystem_f36c58;
  return /*#__PURE__*/React.createElement("div", {
    className: "kit-login"
  }, /*#__PURE__*/React.createElement("div", {
    className: "kit-login__card"
  }, /*#__PURE__*/React.createElement("div", {
    className: "kit-login__brand"
  }, /*#__PURE__*/React.createElement("img", {
    src: LOGIN_BRAND + "hashicorp-color-16.svg",
    alt: "",
    width: "32",
    height: "32"
  }), /*#__PURE__*/React.createElement("span", null, "HashiCorp Cloud Platform")), /*#__PURE__*/React.createElement("h1", {
    className: "kit-login__title"
  }, "Sign in to your account"), /*#__PURE__*/React.createElement("p", {
    className: "kit-login__sub"
  }, "Manage infrastructure, secrets, and networking in one place."), /*#__PURE__*/React.createElement("form", {
    className: "kit-login__form",
    onSubmit: e => {
      e.preventDefault();
      onLogin();
    }
  }, /*#__PURE__*/React.createElement(TextInput, {
    label: "Email",
    type: "email",
    placeholder: "you@company.com",
    defaultValue: "mitchell@hashicorp-demo.com"
  }), /*#__PURE__*/React.createElement(TextInput, {
    label: "Password",
    type: "password",
    placeholder: "\u2022\u2022\u2022\u2022\u2022\u2022\u2022\u2022",
    defaultValue: "terraform"
  }), /*#__PURE__*/React.createElement("div", {
    className: "kit-login__row"
  }, /*#__PURE__*/React.createElement(Checkbox, {
    label: "Remember me",
    defaultChecked: true
  }), /*#__PURE__*/React.createElement("a", {
    href: "#",
    className: "kit-link"
  }, "Forgot password?")), /*#__PURE__*/React.createElement(Button, {
    color: "primary",
    size: "large",
    type: "submit",
    isFullWidth: true
  }, "Sign in")), /*#__PURE__*/React.createElement("div", {
    className: "kit-login__divider"
  }, /*#__PURE__*/React.createElement("span", null, "or")), /*#__PURE__*/React.createElement(Button, {
    color: "secondary",
    size: "large",
    isFullWidth: true,
    icon: /*#__PURE__*/React.createElement(Icon, {
      src: LOGIN_ICN + "lock-16.svg"
    })
  }, "Continue with SSO")), /*#__PURE__*/React.createElement("p", {
    className: "kit-login__legal"
  }, "By signing in you agree to the Terms of Service and Privacy Policy."));
}
window.LoginScreen = LoginScreen;
})(); } catch (e) { __ds_ns.__errors.push({ path: "ui_kits/hcp/LoginScreen.jsx", error: String((e && e.message) || e) }); }

// ui_kits/hcp/OverviewScreen.jsx
try { (() => {
/* global React */
const OV_ICN = "../../assets/icons/";
function OverviewScreen({
  onOpenWorkspaces,
  onOpenWs
}) {
  const {
    Button,
    Badge,
    Icon
  } = window.HeliosDesignSystem_f36c58;
  const ic = n => /*#__PURE__*/React.createElement(Icon, {
    src: OV_ICN + n + ".svg"
  });
  const data = window.KIT_DATA.workspaces.slice(0, 4);
  const SM = window.STATUS_MAP;
  return /*#__PURE__*/React.createElement("div", {
    className: "kit-overview"
  }, /*#__PURE__*/React.createElement("div", {
    className: "kit-hello"
  }, /*#__PURE__*/React.createElement("span", {
    className: "kit-eyebrow"
  }, "Good afternoon \xB7 Today"), /*#__PURE__*/React.createElement("h1", null, "Your fleet is ", /*#__PURE__*/React.createElement("em", null, "healthy."))), /*#__PURE__*/React.createElement("div", {
    className: "kit-hero"
  }, /*#__PURE__*/React.createElement("div", {
    className: "kit-hero__top"
  }, /*#__PURE__*/React.createElement("span", {
    className: "kit-hero__live"
  }, /*#__PURE__*/React.createElement("span", {
    className: "kit-dot"
  }), " Live \xB7 6 workspaces"), /*#__PURE__*/React.createElement("span", {
    className: "kit-hero__meta"
  }, "2 runs in progress")), /*#__PURE__*/React.createElement("span", {
    className: "kit-eyebrow",
    style: {
      display: "block",
      marginBottom: 2
    }
  }, "Apply success \xB7 last 30 days"), /*#__PURE__*/React.createElement("div", {
    className: "kit-hero__stat"
  }, /*#__PURE__*/React.createElement("span", {
    className: "kit-hero__num"
  }, "98.4"), /*#__PURE__*/React.createElement("span", {
    className: "kit-hero__unit"
  }, "%"), /*#__PURE__*/React.createElement(Badge, {
    color: "success",
    icon: /*#__PURE__*/React.createElement(Icon, {
      src: OV_ICN + "check-circle-16.svg",
      size: 14
    }),
    text: "+2.1% vs last month",
    style: {
      marginBottom: 12,
      marginLeft: 4
    }
  })), /*#__PURE__*/React.createElement("div", {
    className: "kit-bar"
  }, /*#__PURE__*/React.createElement("div", {
    className: "kit-bar__fill",
    style: {
      width: "98.4%"
    }
  })), /*#__PURE__*/React.createElement("div", {
    className: "kit-bar__labels"
  }, /*#__PURE__*/React.createElement("span", null, "0"), /*#__PURE__*/React.createElement("span", null, "318 runs \xB7 5 failed")), /*#__PURE__*/React.createElement("div", {
    className: "kit-hero__actions"
  }, /*#__PURE__*/React.createElement(Button, {
    color: "primary",
    icon: ic("plus-16")
  }, "New run"), /*#__PURE__*/React.createElement(Button, {
    color: "secondary",
    icon: ic("clock-16"),
    onClick: onOpenWorkspaces
  }, "Run history"), /*#__PURE__*/React.createElement(Button, {
    color: "tertiary"
  }, "View insights"))), /*#__PURE__*/React.createElement("div", {
    className: "kit-ov-grid"
  }, /*#__PURE__*/React.createElement("div", {
    className: "kit-ov-panel"
  }, /*#__PURE__*/React.createElement("div", {
    className: "kit-ov-panel__head"
  }, /*#__PURE__*/React.createElement("span", {
    className: "kit-eyebrow"
  }, "Workspaces"), /*#__PURE__*/React.createElement("button", {
    className: "kit-textlink",
    onClick: onOpenWorkspaces
  }, "View all")), /*#__PURE__*/React.createElement("div", {
    className: "kit-ov-wslist"
  }, data.map(w => {
    const s = SM[w.status];
    return /*#__PURE__*/React.createElement("button", {
      className: "kit-ov-ws",
      key: w.id,
      onClick: () => onOpenWs(w)
    }, /*#__PURE__*/React.createElement("span", {
      className: "kit-ov-ws__name"
    }, /*#__PURE__*/React.createElement("b", null, w.name), /*#__PURE__*/React.createElement("span", null, w.provider.toUpperCase(), " \xB7 ", w.region)), /*#__PURE__*/React.createElement(Badge, {
      color: s.color,
      size: "small",
      icon: /*#__PURE__*/React.createElement(Icon, {
        src: OV_ICN + s.icon + ".svg",
        size: 14
      }),
      text: s.label
    }), ic("chevron-right-16"));
  }))), /*#__PURE__*/React.createElement("div", {
    className: "kit-ov-panel"
  }, /*#__PURE__*/React.createElement("div", {
    className: "kit-ov-panel__head"
  }, /*#__PURE__*/React.createElement("span", {
    className: "kit-eyebrow"
  }, "Today \xB7 Run time"), /*#__PURE__*/React.createElement("span", {
    className: "kit-ov-pct"
  }, "93% applied")), /*#__PURE__*/React.createElement("div", {
    className: "kit-focusbar"
  }, /*#__PURE__*/React.createElement("i", {
    style: {
      width: "93%"
    }
  })), /*#__PURE__*/React.createElement("div", {
    className: "kit-ov-legend"
  }, /*#__PURE__*/React.createElement("span", null, /*#__PURE__*/React.createElement("span", {
    className: "kit-d-ok"
  }), " 3h 12m applying"), /*#__PURE__*/React.createElement("span", null, /*#__PURE__*/React.createElement("span", {
    className: "kit-d-warn"
  }), " 14m queued")), /*#__PURE__*/React.createElement("div", {
    className: "kit-ov-panel__head",
    style: {
      margin: "20px 0 12px"
    }
  }, /*#__PURE__*/React.createElement("span", {
    className: "kit-eyebrow"
  }, "Providers")), /*#__PURE__*/React.createElement("div", {
    style: {
      display: "flex",
      gap: 8,
      flexWrap: "wrap"
    }
  }, /*#__PURE__*/React.createElement(Badge, {
    type: "outlined",
    color: "neutral",
    text: "AWS \xB7 4"
  }), /*#__PURE__*/React.createElement(Badge, {
    type: "outlined",
    color: "neutral",
    text: "GCP \xB7 1"
  }), /*#__PURE__*/React.createElement(Badge, {
    type: "outlined",
    color: "neutral",
    text: "Azure \xB7 1"
  }), /*#__PURE__*/React.createElement(Badge, {
    color: "highlight",
    text: "Enterprise"
  })))));
}
window.OverviewScreen = OverviewScreen;
})(); } catch (e) { __ds_ns.__errors.push({ path: "ui_kits/hcp/OverviewScreen.jsx", error: String((e && e.message) || e) }); }

// ui_kits/hcp/SideNav.jsx
try { (() => {
/* global React */
const NAV_ICN = "../../assets/icons/";
const NAV_BRAND = "../../assets/brand/";
function SideNav({
  active,
  onNavigate,
  org = "hashicorp-demo"
}) {
  const {
    Icon
  } = window.HeliosDesignSystem_f36c58;
  const ic = n => /*#__PURE__*/React.createElement(Icon, {
    src: NAV_ICN + n + ".svg",
    size: 16
  });
  const items = [{
    id: "overview",
    label: "Overview",
    icon: "dashboard-16"
  }, {
    id: "workspaces",
    label: "Workspaces",
    icon: "layers-16"
  }, {
    id: "registry",
    label: "Registry",
    icon: "folder-16"
  }, {
    id: "runs",
    label: "Runs",
    icon: "clock-16"
  }, {
    id: "settings",
    label: "Settings",
    icon: "settings-16"
  }];
  return /*#__PURE__*/React.createElement("nav", {
    className: "kit-sidenav"
  }, /*#__PURE__*/React.createElement("div", {
    className: "kit-sidenav__brand"
  }, /*#__PURE__*/React.createElement("img", {
    src: NAV_BRAND + "terraform-color-24.svg",
    alt: "",
    width: "22",
    height: "22"
  }), org), /*#__PURE__*/React.createElement("div", {
    className: "kit-sidenav__group"
  }, /*#__PURE__*/React.createElement("span", {
    className: "kit-eyebrow kit-sidenav__label"
  }, "Navigate"), /*#__PURE__*/React.createElement("ul", {
    className: "kit-sidenav__list"
  }, items.map(it => /*#__PURE__*/React.createElement("li", {
    key: it.id
  }, /*#__PURE__*/React.createElement("button", {
    className: "kit-navitem" + (active === it.id ? " kit-navitem--active" : ""),
    onClick: () => onNavigate(it.id)
  }, ic(it.icon), " ", it.label))))), /*#__PURE__*/React.createElement("div", {
    className: "kit-sidenav__spacer"
  }), /*#__PURE__*/React.createElement("span", {
    className: "kit-eyebrow kit-sidenav__label"
  }, "Status"), /*#__PURE__*/React.createElement("div", {
    className: "kit-nowcard"
  }, /*#__PURE__*/React.createElement("div", {
    className: "kit-nowcard__body"
  }, /*#__PURE__*/React.createElement("div", {
    className: "kit-nowcard__num"
  }, "98.4%"), /*#__PURE__*/React.createElement("div", {
    className: "kit-nowcard__sub"
  }, /*#__PURE__*/React.createElement("span", {
    className: "kit-dot"
  }), " Fleet healthy")), ic("server-16")));
}
window.SideNav = SideNav;
})(); } catch (e) { __ds_ns.__errors.push({ path: "ui_kits/hcp/SideNav.jsx", error: String((e && e.message) || e) }); }

// ui_kits/hcp/WorkspaceDetail.jsx
try { (() => {
/* global React */
const WD_ICN = "../../assets/icons/";
function RunRow({
  run,
  onConfirm
}) {
  const {
    Badge,
    Icon,
    Button
  } = window.HeliosDesignSystem_f36c58;
  const SM = window.STATUS_MAP;
  const s = SM[run.status];
  return /*#__PURE__*/React.createElement("div", {
    className: "kit-run"
  }, /*#__PURE__*/React.createElement("div", {
    className: "kit-run__main"
  }, /*#__PURE__*/React.createElement(Badge, {
    color: s.color,
    size: "small",
    icon: /*#__PURE__*/React.createElement(Icon, {
      src: WD_ICN + s.icon + ".svg",
      size: 14
    }),
    text: s.label
  }), /*#__PURE__*/React.createElement("div", {
    className: "kit-run__text"
  }, /*#__PURE__*/React.createElement("span", {
    className: "kit-run__msg"
  }, run.message), /*#__PURE__*/React.createElement("span", {
    className: "kit-run__meta"
  }, /*#__PURE__*/React.createElement("span", {
    className: "kit-run__id"
  }, run.id), " \xB7 ", run.trigger, " \xB7 ", run.author, " \xB7 ", run.time))), /*#__PURE__*/React.createElement("div", {
    className: "kit-run__right"
  }, /*#__PURE__*/React.createElement("span", {
    className: "kit-diff"
  }, /*#__PURE__*/React.createElement("span", {
    className: "kit-diff__add"
  }, "+", run.add), /*#__PURE__*/React.createElement("span", {
    className: "kit-diff__chg"
  }, "~", run.change), /*#__PURE__*/React.createElement("span", {
    className: "kit-diff__del"
  }, "-", run.destroy)), run.status === "needs-confirmation" ? /*#__PURE__*/React.createElement(Button, {
    color: "primary",
    size: "small",
    onClick: onConfirm
  }, "Confirm & apply") : /*#__PURE__*/React.createElement(Icon, {
    src: WD_ICN + "chevron-right-16.svg",
    size: 16
  })));
}
function WorkspaceDetail({
  workspace,
  onBack
}) {
  const {
    Button,
    Badge,
    Icon,
    Tabs,
    Alert,
    Card
  } = window.HeliosDesignSystem_f36c58;
  const SM = window.STATUS_MAP;
  const [tab, setTab] = React.useState("runs");
  const [runs, setRuns] = React.useState(window.KIT_DATA.runs);
  const s = SM[workspace.status];
  const pending = runs.find(r => r.status === "needs-confirmation");
  const confirm = id => setRuns(rs => rs.map(r => r.id === id ? {
    ...r,
    status: "applied",
    time: "just now"
  } : r));
  return /*#__PURE__*/React.createElement("div", {
    className: "kit-page"
  }, /*#__PURE__*/React.createElement("button", {
    className: "kit-back",
    onClick: onBack
  }, /*#__PURE__*/React.createElement(Icon, {
    src: WD_ICN + "chevron-left-16.svg",
    size: 16
  }), " Workspaces"), /*#__PURE__*/React.createElement("div", {
    className: "kit-page__head"
  }, /*#__PURE__*/React.createElement("div", {
    className: "kit-ws-head"
  }, /*#__PURE__*/React.createElement("h1", {
    className: "kit-title"
  }, workspace.name), /*#__PURE__*/React.createElement(Badge, {
    color: s.color,
    icon: /*#__PURE__*/React.createElement(Icon, {
      src: WD_ICN + s.icon + ".svg",
      size: 14
    }),
    text: s.label
  })), /*#__PURE__*/React.createElement("div", {
    className: "kit-page__actions"
  }, /*#__PURE__*/React.createElement(Button, {
    color: "secondary",
    icon: /*#__PURE__*/React.createElement(Icon, {
      src: WD_ICN + "settings-16.svg"
    })
  }, "Settings"), /*#__PURE__*/React.createElement(Button, {
    color: "primary",
    icon: /*#__PURE__*/React.createElement(Icon, {
      src: WD_ICN + "plus-16.svg"
    })
  }, "New run"))), /*#__PURE__*/React.createElement("div", {
    className: "kit-ws-meta"
  }, /*#__PURE__*/React.createElement("span", null, /*#__PURE__*/React.createElement(Icon, {
    src: WD_ICN + "layers-16.svg",
    size: 14
  }), " ", workspace.resources, " resources"), /*#__PURE__*/React.createElement("span", null, /*#__PURE__*/React.createElement(Icon, {
    src: WD_ICN + "clock-16.svg",
    size: 14
  }), " ", workspace.runs, " runs"), /*#__PURE__*/React.createElement("span", null, /*#__PURE__*/React.createElement(Icon, {
    src: WD_ICN + "folder-16.svg",
    size: 14
  }), " ", workspace.project), /*#__PURE__*/React.createElement("span", null, /*#__PURE__*/React.createElement(Icon, {
    src: WD_ICN + "globe-16.svg",
    size: 14
  }), " ", workspace.region)), /*#__PURE__*/React.createElement(Tabs, {
    value: tab,
    onChange: setTab,
    tabs: [{
      id: "overview",
      label: "Overview"
    }, {
      id: "runs",
      label: "Runs",
      count: runs.length
    }, {
      id: "states",
      label: "States"
    }, {
      id: "variables",
      label: "Variables",
      count: 8
    }, {
      id: "settings",
      label: "Settings"
    }]
  }), /*#__PURE__*/React.createElement("div", {
    className: "kit-tabpanel"
  }, tab === "runs" && /*#__PURE__*/React.createElement(React.Fragment, null, pending && /*#__PURE__*/React.createElement(Alert, {
    color: "warning",
    title: "A run needs your confirmation",
    actions: /*#__PURE__*/React.createElement(Button, {
      color: "primary",
      size: "small",
      onClick: () => confirm(pending.id)
    }, "Confirm & apply")
  }, pending.message, " \u2014 ", pending.add, " to add, ", pending.change, " to change."), /*#__PURE__*/React.createElement(Card, {
    level: "base",
    padding: false,
    overflow: "hidden"
  }, /*#__PURE__*/React.createElement("div", {
    className: "kit-runs"
  }, runs.map(r => /*#__PURE__*/React.createElement(RunRow, {
    key: r.id,
    run: r,
    onConfirm: () => confirm(r.id)
  }))))), tab !== "runs" && /*#__PURE__*/React.createElement("div", {
    className: "kit-empty"
  }, /*#__PURE__*/React.createElement(Icon, {
    src: WD_ICN + "file-text-16.svg",
    size: 20
  }), /*#__PURE__*/React.createElement("p", null, "The ", /*#__PURE__*/React.createElement("b", null, tab), " view is not part of this UI-kit recreation."))));
}
window.WorkspaceDetail = WorkspaceDetail;
})(); } catch (e) { __ds_ns.__errors.push({ path: "ui_kits/hcp/WorkspaceDetail.jsx", error: String((e && e.message) || e) }); }

// ui_kits/hcp/WorkspacesScreen.jsx
try { (() => {
/* global React */
const WS_ICN = "../../assets/icons/";
function ProviderMark({
  provider
}) {
  const labels = {
    aws: "AWS",
    gcp: "GCP",
    azure: "Azure"
  };
  const colors = {
    aws: "#ff9900",
    gcp: "#4285f4",
    azure: "#0078d4"
  };
  return /*#__PURE__*/React.createElement("span", {
    className: "kit-provider"
  }, /*#__PURE__*/React.createElement("span", {
    className: "kit-provider__dot",
    style: {
      background: colors[provider] || "#888"
    }
  }), labels[provider] || provider);
}
function WorkspacesScreen({
  onOpen
}) {
  const {
    Button,
    Badge,
    Icon,
    TextInput,
    Tabs
  } = window.HeliosDesignSystem_f36c58;
  const data = window.KIT_DATA.workspaces;
  const SM = window.STATUS_MAP;
  const [q, setQ] = React.useState("");
  const [project, setProject] = React.useState("all");
  const projects = ["all", ...Array.from(new Set(data.map(w => w.project)))];
  const rows = data.filter(w => (project === "all" || w.project === project) && w.name.toLowerCase().includes(q.toLowerCase()));
  return /*#__PURE__*/React.createElement("div", {
    className: "kit-page"
  }, /*#__PURE__*/React.createElement("div", {
    className: "kit-page__head"
  }, /*#__PURE__*/React.createElement("div", null, /*#__PURE__*/React.createElement("h1", {
    className: "kit-title"
  }, "Workspaces"), /*#__PURE__*/React.createElement("p", {
    className: "kit-subtitle"
  }, data.length, " workspaces across 4 projects")), /*#__PURE__*/React.createElement("div", {
    className: "kit-page__actions"
  }, /*#__PURE__*/React.createElement(Button, {
    color: "secondary",
    icon: /*#__PURE__*/React.createElement(Icon, {
      src: WS_ICN + "filter-16.svg"
    })
  }, "Filter"), /*#__PURE__*/React.createElement(Button, {
    color: "primary",
    icon: /*#__PURE__*/React.createElement(Icon, {
      src: WS_ICN + "plus-16.svg"
    })
  }, "New workspace"))), /*#__PURE__*/React.createElement("div", {
    className: "kit-toolbar"
  }, /*#__PURE__*/React.createElement("div", {
    className: "kit-toolbar__search"
  }, /*#__PURE__*/React.createElement(TextInput, {
    icon: /*#__PURE__*/React.createElement(Icon, {
      src: WS_ICN + "search-16.svg"
    }),
    placeholder: "Filter by name\u2026",
    value: q,
    onChange: e => setQ(e.target.value)
  })), /*#__PURE__*/React.createElement(Tabs, {
    defaultValue: "all",
    onChange: setProject,
    tabs: projects.map(p => ({
      id: p,
      label: p === "all" ? "All projects" : p
    }))
  })), /*#__PURE__*/React.createElement("div", {
    className: "kit-table"
  }, /*#__PURE__*/React.createElement("div", {
    className: "kit-table__head"
  }, /*#__PURE__*/React.createElement("div", {
    className: "c-name"
  }, "Name"), /*#__PURE__*/React.createElement("div", {
    className: "c-status"
  }, "Status"), /*#__PURE__*/React.createElement("div", {
    className: "c-provider"
  }, "Provider"), /*#__PURE__*/React.createElement("div", {
    className: "c-res"
  }, "Resources"), /*#__PURE__*/React.createElement("div", {
    className: "c-updated"
  }, "Updated")), rows.map(w => {
    const s = SM[w.status];
    return /*#__PURE__*/React.createElement("button", {
      key: w.id,
      className: "kit-table__row",
      onClick: () => onOpen(w)
    }, /*#__PURE__*/React.createElement("div", {
      className: "c-name"
    }, /*#__PURE__*/React.createElement("span", {
      className: "kit-ws-name"
    }, w.name), /*#__PURE__*/React.createElement("span", {
      className: "kit-ws-project"
    }, w.project)), /*#__PURE__*/React.createElement("div", {
      className: "c-status"
    }, /*#__PURE__*/React.createElement(Badge, {
      color: s.color,
      size: "small",
      icon: /*#__PURE__*/React.createElement(Icon, {
        src: WS_ICN + s.icon + ".svg",
        size: 14
      }),
      text: s.label
    })), /*#__PURE__*/React.createElement("div", {
      className: "c-provider"
    }, /*#__PURE__*/React.createElement(ProviderMark, {
      provider: w.provider
    })), /*#__PURE__*/React.createElement("div", {
      className: "c-res"
    }, w.resources), /*#__PURE__*/React.createElement("div", {
      className: "c-updated"
    }, w.changed, /*#__PURE__*/React.createElement(Icon, {
      src: WS_ICN + "chevron-right-16.svg",
      size: 16
    })));
  })));
}
window.WorkspacesScreen = WorkspacesScreen;
})(); } catch (e) { __ds_ns.__errors.push({ path: "ui_kits/hcp/WorkspacesScreen.jsx", error: String((e && e.message) || e) }); }

// ui_kits/hcp/data.js
try { (() => {
/* Demo data for the HCP Terraform UI kit (fictional) */
window.KIT_DATA = {
  workspaces: [{
    id: "networking-prod",
    name: "networking-prod",
    project: "Platform",
    provider: "aws",
    region: "us-east-1",
    status: "applied",
    resources: 142,
    changed: "2h ago",
    runs: 318
  }, {
    id: "billing-api",
    name: "billing-api",
    project: "Payments",
    provider: "gcp",
    region: "europe-west1",
    status: "planning",
    resources: 64,
    changed: "3m ago",
    runs: 91
  }, {
    id: "data-platform",
    name: "data-platform",
    project: "Data",
    provider: "azure",
    region: "eastus2",
    status: "errored",
    resources: 0,
    changed: "14m ago",
    runs: 27
  }, {
    id: "edge-cdn",
    name: "edge-cdn",
    project: "Platform",
    provider: "aws",
    region: "global",
    status: "needs-confirmation",
    resources: 38,
    changed: "1h ago",
    runs: 204
  }, {
    id: "identity-staging",
    name: "identity-staging",
    project: "Security",
    provider: "aws",
    region: "us-west-2",
    status: "applied",
    resources: 56,
    changed: "yesterday",
    runs: 76
  }, {
    id: "observability",
    name: "observability",
    project: "Platform",
    provider: "gcp",
    region: "us-central1",
    status: "applied",
    resources: 88,
    changed: "yesterday",
    runs: 145
  }],
  runs: [{
    id: "run-8Q2f",
    message: "Bump RDS instance class to r6g.xlarge",
    status: "applied",
    trigger: "merge to main",
    author: "Mitchell Hashimoto",
    time: "2h ago",
    add: 0,
    change: 2,
    destroy: 0
  }, {
    id: "run-7Yx1",
    message: "Add CloudFront distribution for static assets",
    status: "needs-confirmation",
    trigger: "manual run",
    author: "Armon Dadgar",
    time: "1h ago",
    add: 6,
    change: 1,
    destroy: 0
  }, {
    id: "run-7Tg9",
    message: "Rotate NAT gateway EIPs",
    status: "applied",
    trigger: "merge to main",
    author: "Jane Doe",
    time: "yesterday",
    add: 2,
    change: 0,
    destroy: 2
  }, {
    id: "run-7Pb0",
    message: "Refactor VPC module to v4",
    status: "errored",
    trigger: "pull request #482",
    author: "Sam Rivera",
    time: "yesterday",
    add: 0,
    change: 0,
    destroy: 0
  }, {
    id: "run-7Ka3",
    message: "Enable flow logs on private subnets",
    status: "applied",
    trigger: "merge to main",
    author: "Mitchell Hashimoto",
    time: "2 days ago",
    add: 4,
    change: 0,
    destroy: 0
  }]
};
window.STATUS_MAP = {
  applied: {
    color: "success",
    label: "Applied",
    icon: "check-circle-16"
  },
  planning: {
    color: "highlight",
    label: "Planning",
    icon: "loading-16"
  },
  errored: {
    color: "critical",
    label: "Errored",
    icon: "x-circle-16"
  },
  "needs-confirmation": {
    color: "warning",
    label: "Needs confirmation",
    icon: "alert-triangle-16"
  }
};
})(); } catch (e) { __ds_ns.__errors.push({ path: "ui_kits/hcp/data.js", error: String((e && e.message) || e) }); }

__ds_ns.Button = __ds_scope.Button;

__ds_ns.IconButton = __ds_scope.IconButton;

__ds_ns.Card = __ds_scope.Card;

__ds_ns.Alert = __ds_scope.Alert;

__ds_ns.Badge = __ds_scope.Badge;

__ds_ns.Checkbox = __ds_scope.Checkbox;

__ds_ns.Select = __ds_scope.Select;

__ds_ns.TextInput = __ds_scope.TextInput;

__ds_ns.Toggle = __ds_scope.Toggle;

__ds_ns.Avatar = __ds_scope.Avatar;

__ds_ns.Icon = __ds_scope.Icon;

__ds_ns.Tabs = __ds_scope.Tabs;

})();
