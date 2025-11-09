function Frame1() {
  return (
    <div className="content-stretch flex gap-[8px] items-start relative shrink-0">
      <ComponentStatusBadge />
      <ComponentStatusBadge />
    </div>
  );
}

function Frame2() {
  return (
    <div className="content-stretch flex items-start justify-between relative shrink-0 w-full">
      <Frame1 />
      <ComponentStatusBadge />
    </div>
  );
}

function Frame3() {
  return (
    <div className="content-stretch flex flex-col gap-[8px] items-start relative shrink-0 w-full">
      <Frame2 />
      <div className="flex flex-col font-['Inter:Medium',sans-serif] font-medium justify-center leading-[0] not-italic relative shrink-0 text-[24px] text-center text-neutral-50 text-nowrap tracking-[-0.24px]">
        <p className="leading-[normal] whitespace-pre">Task completion</p>
      </div>
    </div>
  );
}

function ComponentStatusBadge() {
  return (
    <div className="relative rounded-[2px] shrink-0" data-name=".component status badge">
      <div className="box-border content-stretch flex items-center overflow-clip px-[4px] py-[2px] relative rounded-[inherit]">
        <div className="w-2 h-2 bg-[#cacaca] rounded" />
        <div className="flex flex-col font-['Inter:Bold',sans-serif] font-bold justify-center leading-[0] not-italic relative shrink-0 text-[#cacaca] text-[8px] text-nowrap text-right tracking-[0.12px]">
          <p className="leading-none whitespace-pre">12 tasks</p>
        </div>
      </div>
      <div aria-hidden="true" className="absolute border border-[#cacaca] border-solid inset-0 pointer-events-none rounded-[2px]" />
    </div>
  );
}

function Frame5() {
  return (
    <div className="content-stretch flex gap-[8px] items-center relative shrink-0">
      <div className="flex flex-col font-['Inter:Light',sans-serif] font-light justify-end leading-[0] not-italic relative shrink-0 text-[48px] text-center text-neutral-50 text-nowrap tracking-[-2.4px]">
        <p className="leading-[48px] whitespace-pre">71%</p>
      </div>
      <ComponentStatusBadge />
      <div className="flex flex-col font-['Inter:Medium',sans-serif] font-medium justify-center leading-[0] not-italic relative shrink-0 text-[#cacaca] text-[10px] text-center text-nowrap tracking-[-0.1px]">
        <p className="leading-[12px] whitespace-pre">since last week</p>
      </div>
    </div>
  );
}

function Frame4() {
  return (
    <div className="content-stretch flex flex-col gap-[8px] items-start relative shrink-0">
      <div className="flex flex-col font-['Inter:Medium',sans-serif] font-medium justify-center leading-[0] not-italic relative shrink-0 text-[#cacaca] text-[10px] text-center text-nowrap tracking-[-0.1px]">
        <p className="leading-[12px] whitespace-pre">You have 12 tasks out of 30 completed</p>
      </div>
      <Frame5 />
    </div>
  );
}

function Frame() {
  return (
    <div className="basis-0 content-stretch flex gap-[2px] grow items-center max-h-[64px] min-h-px min-w-px overflow-clip relative rounded-[4px] shrink-0 w-full">
      <div className="basis-0 bg-neutral-50 grow h-[108px] min-h-px min-w-px shrink-0" />
      <div className="basis-0 bg-neutral-50 grow h-[108px] min-h-px min-w-px shrink-0" />
      <div className="basis-0 bg-neutral-50 grow h-[108px] min-h-px min-w-px shrink-0" />
      <div className="basis-0 bg-neutral-50 grow h-[108px] min-h-px min-w-px shrink-0" />
      <div className="basis-0 bg-neutral-50 grow h-[108px] min-h-px min-w-px shrink-0" />
      <div className="basis-0 bg-neutral-50 grow h-[108px] min-h-px min-w-px shrink-0" />
      <div className="basis-0 bg-neutral-50 grow h-[108px] min-h-px min-w-px shrink-0" />
      <div className="basis-0 bg-neutral-50 grow h-[108px] min-h-px min-w-px shrink-0" />
      <div className="basis-0 bg-neutral-50 grow h-[108px] min-h-px min-w-px shrink-0" />
      <div className="basis-0 bg-neutral-50 grow h-[108px] min-h-px min-w-px shrink-0" />
      <div className="basis-0 bg-neutral-50 grow h-[108px] min-h-px min-w-px shrink-0" />
      <div className="basis-0 bg-neutral-50 grow h-[108px] min-h-px min-w-px shrink-0" />
      <div className="basis-0 bg-neutral-50 grow h-[108px] min-h-px min-w-px shrink-0" />
      <div className="basis-0 bg-neutral-50 grow h-[108px] min-h-px min-w-px shrink-0" />
      <div className="basis-0 bg-neutral-50 grow h-[108px] min-h-px min-w-px shrink-0" />
      <div className="basis-0 bg-neutral-50 grow h-[108px] min-h-px min-w-px shrink-0" />
      <div className="basis-0 bg-neutral-50 grow h-[108px] min-h-px min-w-px shrink-0" />
      <div className="basis-0 bg-neutral-50 grow h-[108px] min-h-px min-w-px shrink-0" />
      <div className="basis-0 bg-neutral-50 grow h-[108px] min-h-px min-w-px shrink-0" />
      <div className="basis-0 bg-[#454545] grow h-[108px] min-h-px min-w-px shrink-0" />
      <div className="basis-0 bg-[#454545] grow h-[108px] min-h-px min-w-px shrink-0" />
      <div className="basis-0 bg-[#454545] grow h-[108px] min-h-px min-w-px shrink-0" />
      <div className="basis-0 bg-[#454545] grow h-[108px] min-h-px min-w-px shrink-0" />
      <div className="basis-0 bg-[#454545] grow h-[108px] min-h-px min-w-px shrink-0" />
      <div className="basis-0 bg-[#454545] grow h-[108px] min-h-px min-w-px shrink-0" />
      <div className="basis-0 bg-[#454545] grow h-[108px] min-h-px min-w-px shrink-0" />
      <div className="basis-0 bg-[#454545] grow h-[108px] min-h-px min-w-px shrink-0" />
      <div className="basis-0 bg-[#454545] grow h-[108px] min-h-px min-w-px shrink-0" />
      <div className="basis-0 bg-[#454545] grow h-[108px] min-h-px min-w-px shrink-0" />
      <div className="basis-0 bg-[#454545] grow h-[108px] min-h-px min-w-px shrink-0" />
      <div className="basis-0 bg-[#454545] grow h-[108px] min-h-px min-w-px shrink-0" />
      <div className="basis-0 bg-[#454545] grow h-[108px] min-h-px min-w-px shrink-0" />
      <div className="basis-0 bg-[#454545] grow h-[108px] min-h-px min-w-px shrink-0" />
      <div className="basis-0 bg-[#454545] grow h-[108px] min-h-px min-w-px shrink-0" />
      <div className="basis-0 bg-[#454545] grow h-[108px] min-h-px min-w-px shrink-0" />
      <div className="basis-0 bg-[#454545] grow h-[108px] min-h-px min-w-px shrink-0" />
      <div className="basis-0 bg-[#454545] grow h-[108px] min-h-px min-w-px shrink-0" />
      <div className="basis-0 bg-[#454545] grow h-[108px] min-h-px min-w-px shrink-0" />
      <div className="basis-0 bg-[#454545] grow h-[108px] min-h-px min-w-px shrink-0" />
      <div className="basis-0 bg-[#454545] grow h-[108px] min-h-px min-w-px shrink-0" />
    </div>
  );
}

export default function Frame6() {
  return (
    <div className="bg-neutral-950 relative rounded-[12px] size-full">
      <div className="size-full">
        <div className="box-border content-stretch flex flex-col gap-[16px] items-start overflow-clip p-[12px] relative size-full">
          <Frame3 />
          <div className="bg-[#cacaca] h-px shrink-0 w-full" />
          <Frame4 />
          <Frame />
        </div>
      </div>
      <div aria-hidden="true" className="absolute border border-[#cacaca] border-solid inset-0 pointer-events-none rounded-[12px]" />
    </div>
  );
}