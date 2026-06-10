import { Check, Code2, X, Zap } from "lucide-react";
import { Button } from "@/src/components/shadcn/button";
import { Badge } from "@/src/components/shadcn/badge";
import { Input } from "@/src/components/shadcn/input";
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/src/components/shadcn/card";

const TOKENS = [
  { name: "primary", className: "bg-primary" },
  { name: "secondary", className: "bg-secondary" },
  { name: "muted", className: "bg-muted" },
  { name: "accent", className: "bg-accent" },
  { name: "destructive", className: "bg-destructive" },
  { name: "border", className: "bg-border" },
];

const CHARTS = ["bg-chart-1", "bg-chart-2", "bg-chart-3", "bg-chart-4", "bg-chart-5"];

export default function Home() {
  return (
    <main className="mx-auto w-full max-w-4xl px-6 py-12">
      <header className="mb-10 flex items-center gap-3">
        <div className="flex size-10 items-center justify-center rounded-lg bg-primary text-primary-foreground">
          <Code2 className="size-5" />
        </div>
        <div>
          <h1 className="text-xl font-semibold tracking-tight">Anvil — theme preview</h1>
          <p className="text-sm text-muted-foreground">Slate + Indigo · shadcn/ui · Tailwind v4</p>
        </div>
      </header>

      <div className="grid gap-6">
        <Card>
          <CardHeader>
            <CardTitle>Buttons</CardTitle>
            <CardDescription>Primary uses the indigo accent; the rest stay neutral.</CardDescription>
          </CardHeader>
          <CardContent className="flex flex-wrap gap-3">
            <Button>Primary</Button>
            <Button variant="secondary">Secondary</Button>
            <Button variant="outline">Outline</Button>
            <Button variant="ghost">Ghost</Button>
            <Button variant="destructive">Destructive</Button>
            <Button variant="link">Link</Button>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Badges, input & test status</CardTitle>
            <CardDescription>Green/red are reserved for pass/fail results.</CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="flex flex-wrap items-center gap-2">
              <Badge>Easy</Badge>
              <Badge variant="secondary">Medium</Badge>
              <Badge variant="outline">Hard</Badge>
              <Badge variant="destructive">Failing</Badge>
            </div>
            <div className="flex flex-wrap items-center gap-4">
              <span className="inline-flex items-center gap-1.5 text-sm font-medium text-chart-2">
                <Check className="size-4" /> 12 / 12 tests passed
              </span>
              <span className="inline-flex items-center gap-1.5 text-sm font-medium text-destructive">
                <X className="size-4" /> 3 / 12 tests failed
              </span>
            </div>
            <Input placeholder="Search problems…" className="max-w-sm" />
          </CardContent>
          <CardFooter className="text-sm text-muted-foreground">
            <Zap className="mr-1.5 size-4 text-primary" /> Toggle OS dark mode to preview the dark
            palette.
          </CardFooter>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Tokens</CardTitle>
            <CardDescription>Semantic surfaces and the chart ramp.</CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="grid grid-cols-3 gap-3 sm:grid-cols-6">
              {TOKENS.map((t) => (
                <div key={t.name} className="space-y-1.5">
                  <div className={`${t.className} h-12 w-full rounded-md border`} />
                  <p className="text-xs text-muted-foreground">{t.name}</p>
                </div>
              ))}
            </div>
            <div className="flex gap-2">
              {CHARTS.map((c) => (
                <div key={c} className={`${c} h-8 flex-1 rounded-md`} />
              ))}
            </div>
          </CardContent>
        </Card>
      </div>
    </main>
  );
}
