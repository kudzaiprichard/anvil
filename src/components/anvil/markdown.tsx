import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { cn } from "@/src/lib/utils";

/**
 * GFM markdown renderer for problem statements, hints, explanations.
 * Typography lives in the `.anvil-prose` rules in globals.css so the same
 * look is reused by the create-form preview (task 0006).
 */
export function Markdown({
  children,
  className,
  id,
  inline,
}: {
  children: string;
  className?: string;
  /** Optional element id, e.g. so an `aria-labelledby` can reference the
   *  rendered prompt as a group label. */
  id?: string;
  /** Render as phrasing content (a `<span>`, paragraphs unwrapped) so the
   *  result is valid inside a `<button>`/label. Use for short one-line labels. */
  inline?: boolean;
}) {
  if (inline) {
    return (
      <span id={id} className={cn("anvil-prose", className)}>
        <ReactMarkdown
          remarkPlugins={[remarkGfm]}
          components={{ p: ({ children }) => <>{children}</> }}
        >
          {children}
        </ReactMarkdown>
      </span>
    );
  }
  return (
    <div id={id} className={cn("anvil-prose", className)}>
      <ReactMarkdown remarkPlugins={[remarkGfm]}>{children}</ReactMarkdown>
    </div>
  );
}
