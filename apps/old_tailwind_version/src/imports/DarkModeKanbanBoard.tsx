import svgPaths from "./svg-ustevohwso";

function Paragraph() {
  return (
    <div className="h-[19.993px] relative shrink-0 w-full" data-name="Paragraph">
      <p className="absolute font-['Inter:Regular',sans-serif] font-normal leading-[20px] left-0 not-italic text-[#888888] text-[14px] text-nowrap top-[0.36px] tracking-[-0.1504px] whitespace-pre">Projects</p>
    </div>
  );
}

function Heading() {
  return (
    <div className="h-[31.996px] relative shrink-0 w-full" data-name="Heading 1">
      <p className="absolute font-['Inter:Regular',sans-serif] font-normal leading-[32px] left-0 not-italic text-[24px] text-nowrap text-white top-[-0.18px] tracking-[0.0703px] whitespace-pre">Project name</p>
    </div>
  );
}

function Container() {
  return (
    <div className="content-stretch flex flex-col gap-[3.999px] h-[55.987px] items-start relative shrink-0 w-full" data-name="Container">
      <Paragraph />
      <Heading />
    </div>
  );
}

function Button() {
  return (
    <div className="absolute h-[35.994px] left-0 top-0 w-[67.678px]" data-name="Button">
      <p className="absolute font-['Inter:Regular',sans-serif] font-normal leading-[24px] left-0 not-italic text-[#888888] text-[16px] text-nowrap top-[-0.73px] tracking-[-0.3125px] whitespace-pre">Overview</p>
    </div>
  );
}

function Button1() {
  return (
    <div className="absolute h-[35.994px] left-[91.68px] top-0 w-[80.881px]" data-name="Button">
      <p className="absolute font-['Inter:Regular',sans-serif] font-normal leading-[24px] left-0 not-italic text-[#888888] text-[16px] text-nowrap top-[-0.73px] tracking-[-0.3125px] whitespace-pre">Workspace</p>
    </div>
  );
}

function Container1() {
  return <div className="absolute bg-white h-[1.996px] left-0 top-[34px] w-[40.895px]" data-name="Container" />;
}

function Button2() {
  return (
    <div className="absolute h-[35.994px] left-[196.56px] top-0 w-[40.895px]" data-name="Button">
      <p className="absolute font-['Inter:Regular',sans-serif] font-normal leading-[24px] left-0 not-italic text-[16px] text-nowrap text-white top-[-0.73px] tracking-[-0.3125px] whitespace-pre">Tasks</p>
      <Container1 />
    </div>
  );
}

function Button3() {
  return (
    <div className="absolute h-[35.994px] left-[261.45px] top-0 w-[60.98px]" data-name="Button">
      <p className="absolute font-['Inter:Regular',sans-serif] font-normal leading-[24px] left-0 not-italic text-[#888888] text-[16px] text-nowrap top-[-0.73px] tracking-[-0.3125px] whitespace-pre">Timeline</p>
    </div>
  );
}

function Button4() {
  return (
    <div className="absolute h-[35.994px] left-[346.43px] top-0 w-[113.48px]" data-name="Button">
      <p className="absolute font-['Inter:Regular',sans-serif] font-normal leading-[24px] left-0 not-italic text-[#888888] text-[16px] text-nowrap top-[-0.73px] tracking-[-0.3125px] whitespace-pre">Manage Project</p>
    </div>
  );
}

function Container2() {
  return (
    <div className="h-[35.994px] relative shrink-0 w-[459.908px]" data-name="Container">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border h-[35.994px] relative w-[459.908px]">
        <Button />
        <Button1 />
        <Button2 />
        <Button3 />
        <Button4 />
      </div>
    </div>
  );
}

function Icon() {
  return (
    <div className="absolute left-[96.19px] size-[15.994px] top-[10px]" data-name="Icon">
      <svg className="block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 16 16">
        <g id="Icon">
          <path d={svgPaths.p10a02b40} id="Vector" stroke="var(--stroke-0, white)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
        </g>
      </svg>
    </div>
  );
}

function Button5() {
  return (
    <div className="absolute bg-[#1a1a1a] h-[35.994px] left-[267.99px] rounded-[8px] top-0 w-[125.092px]" data-name="Button">
      <div aria-hidden="true" className="absolute border-[0.909px] border-neutral-800 border-solid inset-0 pointer-events-none rounded-[8px]" />
      <p className="absolute font-['Inter:Medium',sans-serif] font-medium leading-[20px] left-[12.9px] not-italic text-[14px] text-nowrap text-white top-[8.37px] tracking-[-0.1504px] whitespace-pre">Status: All</p>
      <Icon />
    </div>
  );
}

function Icon1() {
  return (
    <div className="absolute left-[12.9px] size-[15.994px] top-[10px]" data-name="Icon">
      <svg className="block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 16 16">
        <g id="Icon">
          <path d={svgPaths.p26dba700} id="Vector" stroke="var(--stroke-0, white)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
          <path d="M11.3293 13.3286V2.66572" id="Vector_2" stroke="var(--stroke-0, white)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
          <path d={svgPaths.pea98c00} id="Vector_3" stroke="var(--stroke-0, white)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
          <path d="M4.66501 2.66572V13.3286" id="Vector_4" stroke="var(--stroke-0, white)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
        </g>
      </svg>
    </div>
  );
}

function Button6() {
  return (
    <div className="absolute bg-[#1a1a1a] h-[35.994px] left-[405.08px] rounded-[8px] top-0 w-[85.76px]" data-name="Button">
      <div aria-hidden="true" className="absolute border-[0.909px] border-neutral-800 border-solid inset-0 pointer-events-none rounded-[8px]" />
      <Icon1 />
      <p className="absolute font-['Inter:Medium',sans-serif] font-medium leading-[20px] left-[44.89px] not-italic text-[14px] text-nowrap text-white top-[8.37px] tracking-[-0.1504px] whitespace-pre">Sort</p>
    </div>
  );
}

function Icon2() {
  return (
    <div className="relative shrink-0 size-[15.994px]" data-name="Icon">
      <svg className="block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 16 16">
        <g id="Icon">
          <path d={svgPaths.p3cc8d400} id="Vector" stroke="var(--stroke-0, white)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
          <path d="M1.99929 5.99787H13.995" id="Vector_2" stroke="var(--stroke-0, white)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
          <path d="M1.99929 9.99645H13.995" id="Vector_3" stroke="var(--stroke-0, white)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
          <path d="M5.99787 1.99929V13.995" id="Vector_4" stroke="var(--stroke-0, white)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
          <path d="M9.99645 1.99929V13.995" id="Vector_5" stroke="var(--stroke-0, white)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
        </g>
      </svg>
    </div>
  );
}

function Button7() {
  return (
    <div className="absolute bg-[#1a1a1a] box-border content-stretch flex h-[35.994px] items-center justify-center left-[502.83px] p-[0.909px] rounded-[8px] top-0 w-[41.804px]" data-name="Button">
      <div aria-hidden="true" className="absolute border-[0.909px] border-neutral-800 border-solid inset-0 pointer-events-none rounded-[8px]" />
      <Icon2 />
    </div>
  );
}

function Input() {
  return (
    <div className="absolute bg-[#1a1a1a] h-[35.994px] left-0 rounded-[8px] top-0 w-[255.994px]" data-name="Input">
      <div className="box-border content-stretch flex h-[35.994px] items-center overflow-clip pl-[36px] pr-[48px] py-[4px] relative rounded-[inherit] w-[255.994px]">
        <p className="font-['Inter:Regular',sans-serif] font-normal leading-[normal] not-italic relative shrink-0 text-[#888888] text-[14px] text-nowrap tracking-[-0.1504px] whitespace-pre">Search</p>
      </div>
      <div aria-hidden="true" className="absolute border-[0.909px] border-neutral-800 border-solid inset-0 pointer-events-none rounded-[8px]" />
    </div>
  );
}

function Icon3() {
  return (
    <div className="absolute left-[12px] size-[15.994px] top-[10px]" data-name="Icon">
      <svg className="block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 16 16">
        <g id="Icon">
          <path d={svgPaths.p24791400} id="Vector" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
          <path d={svgPaths.p2139fb00} id="Vector_2" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
        </g>
      </svg>
    </div>
  );
}

function Text() {
  return (
    <div className="absolute h-[16.001px] left-[226.53px] top-[10px] w-[17.465px]" data-name="Text">
      <p className="absolute font-['Inter:Regular',sans-serif] font-normal leading-[16px] left-0 not-italic text-[#888888] text-[12px] text-nowrap top-[0.46px] whitespace-pre">⌘F</p>
    </div>
  );
}

function Container3() {
  return (
    <div className="absolute h-[35.994px] left-0 top-0 w-[255.994px]" data-name="Container">
      <Input />
      <Icon3 />
      <Text />
    </div>
  );
}

function Container4() {
  return (
    <div className="h-[35.994px] relative shrink-0 w-[544.638px]" data-name="Container">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border h-[35.994px] relative w-[544.638px]">
        <Button5 />
        <Button6 />
        <Button7 />
        <Container3 />
      </div>
    </div>
  );
}

function Container5() {
  return (
    <div className="content-stretch flex h-[35.994px] items-center justify-between relative shrink-0 w-full" data-name="Container">
      <Container2 />
      <Container4 />
    </div>
  );
}

function Container6() {
  return (
    <div className="h-[140.874px] relative shrink-0 w-[2653.64px]" data-name="Container">
      <div aria-hidden="true" className="absolute border-[0px_0px_0.909px] border-neutral-800 border-solid inset-0 pointer-events-none" />
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border content-stretch flex flex-col gap-[15.994px] h-[140.874px] items-start pb-[0.909px] pl-[23.999px] pr-[23.998px] pt-[15.994px] relative w-[2653.64px]">
        <Container />
        <Container5 />
      </div>
    </div>
  );
}

export default function DarkModeKanbanBoard() {
  return (
    <div className="bg-[#0d0d0d] content-stretch flex flex-col items-start relative size-full" data-name="Dark Mode Kanban Board">
      <Container6 />
    </div>
  );
}