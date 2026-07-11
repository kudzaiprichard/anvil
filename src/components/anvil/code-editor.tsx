"use client";

import { useEffect, useRef } from "react";
import { EditorView, basicSetup } from "codemirror";
import { EditorState, Prec } from "@codemirror/state";
import { keymap, scrollPastEnd } from "@codemirror/view";
import { indentWithTab } from "@codemirror/commands";
import {
  HighlightStyle,
  indentUnit,
  syntaxHighlighting,
} from "@codemirror/language";
import { python } from "@codemirror/lang-python";
import { javascript } from "@codemirror/lang-javascript";
import { tags as t } from "@lezer/highlight";
import { cn } from "@/src/lib/utils";
import type { Language } from "@/src/lib/types";

/**
 * CodeMirror 6 wrapper (UI_SPEC §6.2). All colors come from CSS variables in
 * globals.css, so the editor follows the light/dark theme automatically and
 * never falls back to CodeMirror's default (off-theme) palette.
 */

const highlight = HighlightStyle.define([
  {
    tag: [
      t.keyword,
      t.controlKeyword,
      t.moduleKeyword,
      t.operatorKeyword,
      t.definitionKeyword,
      t.self,
      t.null,
    ],
    color: "var(--code-keyword)",
  },
  {
    // types, classes, and built-in constructors (list, Map, …)
    tag: [t.typeName, t.className, t.namespace, t.standard(t.name)],
    color: "var(--code-type)",
  },
  {
    tag: [t.function(t.variableName), t.function(t.propertyName)],
    color: "var(--code-function)",
  },
  {
    tag: [t.string, t.special(t.string), t.regexp, t.character],
    color: "var(--code-string)",
  },
  { tag: [t.escape], color: "var(--code-string)", fontWeight: "600" },
  {
    tag: [t.number, t.integer, t.float, t.bool, t.atom, t.unit],
    color: "var(--code-number)",
  },
  {
    tag: [t.comment, t.lineComment, t.blockComment, t.docComment],
    color: "var(--code-comment)",
    fontStyle: "italic",
  },
  { tag: t.definition(t.variableName), color: "var(--code-param)" },
  { tag: [t.propertyName, t.attributeName], color: "var(--foreground)" },
  {
    tag: [t.operator, t.derefOperator, t.punctuation, t.separator, t.bracket],
    color: "var(--code-operator)",
  },
  { tag: [t.invalid], color: "var(--fail)" },
]);

function buildTheme(fontSize: number, readOnly: boolean) {
  return EditorView.theme({
    "&": {
      backgroundColor: "var(--editor)",
      color: "var(--foreground)",
      fontSize: `${fontSize}px`,
      height: "100%",
    },
    ".cm-scroller": {
      fontFamily: "var(--font-jetbrains-mono), ui-monospace, monospace",
      lineHeight: "1.77",
      paddingTop: "10px",
    },
    ".cm-content": { caretColor: "var(--foreground)" },
    ".cm-gutters": {
      backgroundColor: "var(--editor)",
      color: "var(--editor-gutter)",
      border: "none",
      paddingLeft: "8px",
    },
    ".cm-lineNumbers .cm-gutterElement": {
      paddingRight: "16px",
      minWidth: "38px",
    },
    // No editable affordances when this is a read-only solution viewer.
    ".cm-activeLine": {
      backgroundColor: readOnly ? "transparent" : "var(--editor-linehl)",
    },
    ".cm-activeLineGutter": {
      backgroundColor: "transparent",
      color: readOnly ? "var(--editor-gutter)" : "var(--foreground)",
    },
    ".cm-matchingBracket, &.cm-focused .cm-matchingBracket": {
      backgroundColor: "color-mix(in oklab, var(--primary) 22%, transparent)",
      color: "inherit",
      outline: "none",
    },
    "&.cm-focused": { outline: "none" },
    ".cm-selectionBackground, &.cm-focused .cm-selectionBackground": {
      backgroundColor:
        "color-mix(in oklab, var(--primary) 20%, transparent) !important",
    },
    ".cm-cursor": { borderLeftColor: "var(--foreground)" },

    /* IDE surfaces — search panel, tooltips, autocomplete, folding. The
       defaults ship browser-gray chrome that instantly breaks the theme. */
    ".cm-panels": {
      backgroundColor: "var(--card)",
      color: "var(--foreground)",
    },
    ".cm-panels.cm-panels-top": { borderBottom: "1px solid var(--border)" },
    ".cm-panels.cm-panels-bottom": { borderTop: "1px solid var(--border)" },
    ".cm-panel.cm-search": {
      padding: "6px 10px",
      fontFamily: "var(--font-spline-sans), system-ui, sans-serif",
      fontSize: "12px",
    },
    ".cm-panel.cm-gotoLine": { padding: "6px 10px", fontSize: "12px" },
    ".cm-textfield": {
      backgroundColor: "var(--editor)",
      border: "1px solid var(--input)",
      borderRadius: "6px",
      padding: "3px 8px",
      fontFamily: "var(--font-jetbrains-mono), ui-monospace, monospace",
      fontSize: "12px",
    },
    ".cm-button": {
      backgroundImage: "none",
      backgroundColor: "var(--secondary)",
      color: "var(--secondary-foreground)",
      border: "1px solid var(--border)",
      borderRadius: "6px",
      padding: "3px 10px",
      fontSize: "12px",
    },
    ".cm-button:active": {
      backgroundImage: "none",
      backgroundColor: "var(--accent)",
    },
    ".cm-panel.cm-search label": { fontSize: "12px" },
    ".cm-panel.cm-search input[type=checkbox]": { accentColor: "var(--primary)" },
    ".cm-panel.cm-search [name=close]": {
      color: "var(--muted-foreground)",
      fontSize: "16px",
      padding: "0 6px",
    },
    ".cm-searchMatch": {
      backgroundColor: "color-mix(in oklab, var(--medium) 28%, transparent)",
      outline: "1px solid color-mix(in oklab, var(--medium) 45%, transparent)",
    },
    ".cm-searchMatch.cm-searchMatch-selected": {
      backgroundColor: "color-mix(in oklab, var(--primary) 38%, transparent)",
    },
    ".cm-selectionMatch": {
      backgroundColor: "color-mix(in oklab, var(--primary) 12%, transparent)",
    },
    ".cm-tooltip": {
      backgroundColor: "var(--popover)",
      color: "var(--popover-foreground)",
      border: "1px solid var(--border)",
      borderRadius: "8px",
      boxShadow: "0 8px 24px rgb(0 0 0 / 0.14)",
      overflow: "hidden",
    },
    ".cm-tooltip.cm-tooltip-autocomplete > ul": {
      fontFamily: "var(--font-jetbrains-mono), ui-monospace, monospace",
      fontSize: "12px",
      maxHeight: "224px",
    },
    ".cm-tooltip.cm-tooltip-autocomplete > ul > li": {
      padding: "2.5px 8px",
    },
    ".cm-tooltip-autocomplete ul li[aria-selected]": {
      backgroundColor: "color-mix(in oklab, var(--primary) 15%, transparent)",
      color: "var(--foreground)",
    },
    ".cm-completionIcon": { color: "var(--muted-foreground)" },
    ".cm-completionMatchedText": {
      textDecoration: "none",
      color: "var(--primary)",
      fontWeight: "600",
    },
    ".cm-completionDetail": {
      color: "var(--muted-foreground)",
      fontStyle: "normal",
      marginLeft: "0.6em",
    },
    ".cm-foldPlaceholder": {
      backgroundColor: "var(--muted)",
      border: "none",
      color: "var(--muted-foreground)",
      borderRadius: "4px",
      padding: "0 6px",
      margin: "0 3px",
    },
    ".cm-foldGutter .cm-gutterElement": { color: "var(--editor-gutter)" },
  });
}

/**
 * Chords the workspace window handler owns: Ctrl+[ / Ctrl+] navigate
 * problems (CodeMirror would indent-less/more) and Ctrl+Enter runs the tests
 * (CodeMirror would insert a blank line). Consume them at the highest
 * precedence as no-ops so the editor doesn't also act — the event still
 * bubbles to the window-level `useWorkspaceShortcuts` handler.
 */
const suppressIndentChordsExt = Prec.highest(
  keymap.of([
    { key: "Ctrl-[", run: () => true },
    { key: "Ctrl-]", run: () => true },
    { key: "Ctrl-Enter", run: () => true },
  ])
);

export function CodeEditor({
  value,
  language,
  onChange,
  readOnly = false,
  fontSize = 13,
  tabSize = 4,
  lineWrap = false,
  /** Changing this rebuilds the editor with a fresh undo history — pass the
   *  problem id in the workspace so Ctrl+Z can't cross problems. */
  docKey,
  /** Let window-level shortcuts own Ctrl+[ / Ctrl+] / Ctrl+Enter (workspace only). */
  suppressIndentChords = false,
  className,
}: {
  value: string;
  language: Language;
  onChange?: (code: string) => void;
  readOnly?: boolean;
  fontSize?: number;
  tabSize?: 2 | 4;
  lineWrap?: boolean;
  docKey?: string;
  suppressIndentChords?: boolean;
  className?: string;
}) {
  const hostRef = useRef<HTMLDivElement>(null);
  const viewRef = useRef<EditorView | null>(null);
  const onChangeRef = useRef(onChange);
  useEffect(() => {
    onChangeRef.current = onChange;
  });

  // (Re)create the editor when configuration — or the document identity
  // (`docKey`) — changes. A fresh EditorState means a fresh undo history.
  useEffect(() => {
    if (!hostRef.current) return;
    const extensions = [
      basicSetup,
      keymap.of([indentWithTab]),
      language === "python" ? python() : javascript(),
      indentUnit.of(" ".repeat(tabSize)),
      EditorState.tabSize.of(tabSize),
      buildTheme(fontSize, readOnly),
      syntaxHighlighting(highlight),
      EditorView.updateListener.of((update) => {
        if (update.docChanged) {
          onChangeRef.current?.(update.state.doc.toString());
        }
      }),
    ];
    if (suppressIndentChords) extensions.push(suppressIndentChordsExt);
    if (lineWrap) extensions.push(EditorView.lineWrapping);
    if (!readOnly) {
      // IDE niceties: Alt+click drops an extra cursor (VS Code / JetBrains
      // muscle memory; Alt+drag already does rectangular selection), and the
      // last line can scroll up out of the bottom edge.
      extensions.push(
        EditorView.clickAddsSelectionRange.of((e) => e.altKey && !e.shiftKey)
      );
      extensions.push(scrollPastEnd());
    }
    if (readOnly) {
      // readOnly blocks edits; non-editable also drops the caret so the
      // solution viewer reads as a viewer, not an empty text box.
      extensions.push(EditorState.readOnly.of(true));
      extensions.push(EditorView.editable.of(false));
    }

    const view = new EditorView({
      state: EditorState.create({ doc: value, extensions }),
      parent: hostRef.current,
    });
    viewRef.current = view;
    return () => {
      view.destroy();
      viewRef.current = null;
    };
    // `value` is intentionally not a dependency: external value changes are
    // synced below without rebuilding the editor (which would lose focus).
    // `docKey` IS a dependency so switching problems starts fresh history.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [language, readOnly, fontSize, tabSize, lineWrap, suppressIndentChords, docKey]);

  // Sync external value changes within the same document (reset-to-starter).
  useEffect(() => {
    const view = viewRef.current;
    if (!view) return;
    const current = view.state.doc.toString();
    if (current !== value) {
      view.dispatch({
        changes: { from: 0, to: current.length, insert: value },
      });
    }
  }, [value]);

  return (
    <div
      ref={hostRef}
      className={cn("h-full min-h-0 overflow-hidden bg-editor", className)}
    />
  );
}
