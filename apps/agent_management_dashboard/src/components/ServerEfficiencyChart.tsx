import { BentoPanel } from './compounds';
import { ArrowUpRight } from 'lucide-react';

interface ServerEfficiencyChartProps {
  title?: string;
  efficiency?: number;
}

export function ServerEfficiencyChart({
  title = "Server Efficiency Analysis",
  efficiency = 55
}: ServerEfficiencyChartProps) {
  // Generate bar data with varying heights
  const bars = [
    { height: 30 },
    { height: 45 },
    { height: 65 },
    { height: 50 },
    { height: 85 },
    { height: 40 },
    { height: 70 },
    { height: 90 }
  ];

  return (
    <BentoPanel className="col-span-4 row-span-2">
      <div className="h-full flex flex-col p-6">
        {/* Header */}
        <div className="flex items-start justify-between mb-8">
          <h3 className="text-zinc-400 text-sm">{title}</h3>
          <button className="text-zinc-500 hover:text-zinc-300 transition-colors">
            <ArrowUpRight className="w-4 h-4" />
          </button>
        </div>

        {/* Efficiency Metric */}
        <div className="mb-6">
          <div className="text-white text-4xl">
            +{efficiency}%
          </div>
        </div>

        {/* Bar Chart */}
        <div className="flex-1 flex items-end gap-[6px] justify-between">
          {bars.map((bar, index) => (
            <div
              key={index}
              className="flex-1 bg-zinc-700 rounded-t-sm relative"
              style={{ height: `${bar.height}%` }}
            >
              <div className="absolute top-0 left-0 right-0 h-[3px] bg-zinc-300 rounded-t-sm" />
            </div>
          ))}
        </div>
      </div>
    </BentoPanel>
  );
}
