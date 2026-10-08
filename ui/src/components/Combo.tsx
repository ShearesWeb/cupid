// Combo.tsx — search-as-you-type picker (design combo): a text field that
// lists up to six matches while typing and collapses to a removable chip once
// an option is picked.
import { useState } from "react";
import { Icon } from "./Icon.tsx";
import { PositionTypeBadge } from "./PositionTypeBadge.tsx";
import { TextInput } from "./TextInput.tsx";

export interface ComboOption {
  id: number;
  label: string;
  sub?: string;
  badge?: "block" | "main" | "sub";
}

export function Combo({
  placeholder,
  options,
  selected,
  onSelect,
}: {
  placeholder: string;
  options: (q: string) => ComboOption[];
  selected: ComboOption | null;
  onSelect: (o: ComboOption | null) => void;
}) {
  const [query, setQuery] = useState("");
  const [open, setOpen] = useState(false);
  const matches = open && !selected ? options(query).slice(0, 6) : [];
  return (
    <div style={{ position: "relative", flex: 1, minWidth: 220 }}>
      {selected ? (
        <button
          onClick={() => {
            onSelect(null);
            setQuery("");
          }}
          style={{
            display: "flex",
            alignItems: "center",
            gap: 8,
            width: "100%",
            height: 34,
            padding: "0 10px",
            borderRadius: 7,
            border: "1px solid var(--token-color-border-action)",
            background: "var(--token-color-surface-action)",
            color: "var(--token-color-foreground-action)",
            cursor: "pointer",
            font: "inherit",
            fontSize: 12.5,
            fontWeight: 600,
          }}
        >
          <span style={{ flex: 1, textAlign: "left", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
            {selected.label}
          </span>
          <Icon name="x" size={14} color="var(--token-color-foreground-action)" />
        </button>
      ) : (
        <div onBlur={() => setTimeout(() => setOpen(false), 120)}>
          <TextInput
            placeholder={placeholder}
            icon={<Icon name="search" size={16} color="var(--token-color-foreground-faint)" />}
            value={query}
            onChange={(e) => {
              setQuery(e.target.value);
              // Open only while the operator is typing; an empty field
              // (including right after a grant resets the form) stays closed.
              setOpen(e.target.value.trim().length > 0);
            }}
          />
        </div>
      )}
      {matches.length > 0 ? (
        <div
          style={{
            position: "absolute",
            top: 38,
            left: 0,
            right: 0,
            zIndex: 40,
            borderRadius: 9,
            background: "var(--token-color-surface-primary)",
            boxShadow: "var(--token-elevation-high-box-shadow)",
            border: "1px solid var(--token-color-border-faint)",
            overflow: "hidden",
          }}
        >
          {matches.map((o) => (
            <button
              key={o.id}
              onMouseDown={(e) => {
                e.preventDefault();
                onSelect(o);
                setOpen(false);
                setQuery("");
              }}
              style={{
                display: "flex",
                alignItems: "center",
                gap: 8,
                width: "100%",
                padding: "8px 11px",
                border: "none",
                borderBottom: "1px solid var(--token-color-border-faint)",
                background: "transparent",
                cursor: "pointer",
                font: "inherit",
                fontSize: 12.5,
                textAlign: "left",
              }}
            >
              <span style={{ fontWeight: 600, color: "var(--token-color-foreground-strong)" }}>{o.label}</span>
              {o.badge ? <PositionTypeBadge type={o.badge} /> : null}
              <span
                style={{
                  marginLeft: "auto",
                  color: "var(--token-color-foreground-faint)",
                  fontSize: 11.5,
                  overflow: "hidden",
                  textOverflow: "ellipsis",
                  whiteSpace: "nowrap",
                  maxWidth: "55%",
                }}
              >
                {o.sub}
              </span>
            </button>
          ))}
        </div>
      ) : null}
    </div>
  );
}
