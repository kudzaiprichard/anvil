import { cn } from "@/src/lib/utils";
import type { ActivityDay } from "@/src/lib/types";

/** Indigo intensity ramp (single hue per UI_SPEC §6.1). */
const LEVEL_CLASS = [
  "bg-muted",
  "bg-primary/25",
  "bg-primary/45",
  "bg-primary/70",
  "bg-primary",
] as const;

function level(count: number): number {
  if (count <= 0) return 0;
  if (count === 1) return 1;
  if (count === 2) return 2;
  if (count === 3) return 3;
  return 4;
}

/** GitHub-style activity calendar: columns are weeks, rows are days. */
export function ActivityHeatmap({ activity }: { activity: ActivityDay[] }) {
  const weeks: ActivityDay[][] = [];
  for (let i = 0; i < activity.length; i += 7) {
    weeks.push(activity.slice(i, i + 7));
  }
  return (
    <div>
      <div className="mt-3.5 flex gap-[3px] overflow-hidden">
        {weeks.map((week, w) => (
          <div key={w} className="flex flex-col gap-[3px]">
            {week.map((day) => (
              <div
                key={day.date}
                title={`${day.date} · ${day.count} solved`}
                className={cn(
                  "size-[11px] rounded-[3px]",
                  LEVEL_CLASS[level(day.count)]
                )}
              />
            ))}
          </div>
        ))}
      </div>
      <div className="mt-3.5 flex items-center justify-end gap-1.5">
        <span className="text-[11px] text-muted-foreground">Less</span>
        {LEVEL_CLASS.map((cls, i) => (
          <div key={i} className={cn("size-2.5 rounded-[2.5px]", cls)} />
        ))}
        <span className="text-[11px] text-muted-foreground">More</span>
      </div>
    </div>
  );
}
