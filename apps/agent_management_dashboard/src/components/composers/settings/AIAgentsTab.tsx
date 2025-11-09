'use client';

export function AIAgentsTabContent() {
  return (
    <div className="min-h-[600px] p-[31.996px]">
      <div className="bg-[#1a1a1a] rounded-[10px] p-[24.908px] border-[0.909px] border-neutral-800">
        <h2 className="font-['Inter:Regular',sans-serif] text-[20px] leading-[28px] text-white tracking-[-0.4492px] mb-[15.994px]">
          AI Agents
        </h2>
        <p className="font-['Inter:Regular',sans-serif] text-[14px] leading-[20px] text-[#888888] tracking-[-0.1504px] mb-[31.996px]">
          Configure AI agents to automate tasks and provide intelligent
          assistance.
        </p>

        <div className="space-y-[15.994px]">
          {[
            {
              name: 'Task Suggester',
              description:
                'Automatically suggests task breakdowns and subtasks',
              enabled: true,
            },
            {
              name: 'Priority Optimizer',
              description: 'Analyzes and recommends task prioritization',
              enabled: true,
            },
            {
              name: 'Deadline Predictor',
              description:
                'Estimates realistic completion dates based on history',
              enabled: false,
            },
          ].map((agent, i) => (
            <div
              key={i}
              className="bg-[#0d0d0d] rounded-[8px] p-[16px] border-[0.909px] border-neutral-800 flex items-center justify-between"
            >
              <div>
                <p className="font-['Inter:Medium',sans-serif] text-[14px] text-white tracking-[-0.1504px] mb-[4px]">
                  {agent.name}
                </p>
                <p className="font-['Inter:Regular',sans-serif] text-[14px] text-[#888888] tracking-[-0.1504px]">
                  {agent.description}
                </p>
              </div>
              <div
                className={`h-[18.395px] w-[31.996px] rounded-[1.5252e+07px] ${
                  agent.enabled ? 'bg-[#030213]' : 'bg-[#cbced4]'
                }`}
              >
                <div
                  className={`bg-white relative rounded-[1.5252e+07px] shrink-0 size-[15.994px] transition-transform ${
                    agent.enabled
                      ? 'translate-x-[14.903px]'
                      : 'translate-x-[0.909px]'
                  } mt-[0.909px] ${
                    agent.enabled ? 'ml-[0.909px]' : 'ml-[0.909px]'
                  }`}
                />
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

