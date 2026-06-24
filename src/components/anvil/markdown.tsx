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
}: {
  children: string;
  className?: string;
}) {
  return (
    <div className={cn("anvil-prose", className)}>
      <ReactMarkdown remarkPlugins={[remarkGfm]}>{children}</ReactMarkdown>
    </div>
  );
}
