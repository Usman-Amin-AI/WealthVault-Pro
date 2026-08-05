import { describe, expect, it } from "vitest";
import { normalizeReturnsForChart } from "@/lib/indicators";

describe("normalizeReturnsForChart", () => {
  it("sorts, deduplicates, and ignores malformed timing data for chart rendering", () => {
    const returns = [
      { date: "2024-03-02T00:00:00Z", value: 10 },
      { date: "2024-03-01T00:00:00Z", value: 5 },
      { date: "2024-03-01T00:00:00Z", value: 7 },
      { date: "not-a-date", value: 99 },
      { date: "2024-02-28T00:00:00Z", value: 3 },
    ];

    expect(normalizeReturnsForChart(returns)).toEqual([
      { time: "2024-02-28", value: 3 },
      { time: "2024-03-01", value: 7 },
      { time: "2024-03-02", value: 10 },
    ]);
  });
});
