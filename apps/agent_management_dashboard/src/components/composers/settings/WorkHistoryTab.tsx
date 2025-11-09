'use client';

export function WorkHistoryTabContent() {
  return (
    <div className="min-h-[600px] p-[31.996px]">
      <div className="bg-[#1a1a1a] rounded-[10px] p-[24.908px] border-[0.909px] border-neutral-800">
        <h2 className="font-['Inter:Regular',sans-serif] text-[20px] leading-[28px] text-white tracking-[-0.4492px] mb-[15.994px]">
          Work History
        </h2>
        <p className="font-['Inter:Regular',sans-serif] text-[14px] leading-[20px] text-[#888888] tracking-[-0.1504px]">
          View and analyze your team&apos;s work history, time tracking, and
          productivity metrics.
        </p>
        <div className="mt-[31.996px] grid grid-cols-3 gap-[15.994px]">
          {[
            'Total Tasks',
            'Completed This Week',
            'Average Completion Time',
          ].map((metric, i) => (
            <div
              key={i}
              className="bg-[#0d0d0d] rounded-[8px] p-[16px] border-[0.909px] border-neutral-800"
            >
              <p className="font-['Inter:Medium',sans-serif] text-[14px] text-[#d1d5dc] tracking-[-0.1504px] mb-[8px]">
                {metric}
              </p>
              <p className="font-['Inter:Regular',sans-serif] text-[24px] text-white">
                {i === 0 ? '127' : i === 1 ? '23' : '2.3 days'}
              </p>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

