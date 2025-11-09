import svgPaths from "./svg-23263o4pzr";

interface HeadingProps {
  title: string;
}

function Heading({ title }: HeadingProps) {
  return (
    <div className="h-[23.991px] relative shrink-0" data-name="Heading 3">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border h-[23.991px] relative">
        <p className="font-['Inter:Regular',sans-serif] font-normal leading-[24px] not-italic text-[16px] text-white tracking-[-0.3125px]">{title}</p>
      </div>
    </div>
  );
}

function Icon() {
  return (
    <div className="relative shrink-0 size-[15.994px]" data-name="Icon">
      <svg className="block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 16 16">
        <g id="Icon">
          <path d={svgPaths.p139f1200} id="Vector" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
          <path d={svgPaths.pc092300} id="Vector_2" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
        </g>
      </svg>
    </div>
  );
}

interface ButtonProps {
  onClick: () => void;
}

function Button({ onClick }: ButtonProps) {
  return (
    <button onClick={onClick} className="relative rounded-[8px] shrink-0 size-[31.989px] hover:bg-[#252525] transition-colors" data-name="Button">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border content-stretch flex items-center justify-center relative size-[31.989px]">
        <Icon />
      </div>
    </button>
  );
}

interface ContainerProps {
  title: string;
  onClose: () => void;
}

function Container({ title, onClose }: ContainerProps) {
  return (
    <div className="h-[64.886px] relative shrink-0 w-full" data-name="Container">
      <div aria-hidden="true" className="absolute border-[0px_0px_0.909px] border-neutral-800 border-solid inset-0 pointer-events-none" />
      <div className="flex flex-row items-center size-full">
        <div className="box-border content-stretch flex h-[64.886px] items-center justify-between pb-[0.909px] pl-[23.991px] pr-[23.992px] pt-0 relative w-full">
          <Heading title={title} />
          <Button onClick={onClose} />
        </div>
      </div>
    </div>
  );
}

function Icon1() {
  return (
    <div className="relative shrink-0 size-[31.989px]" data-name="Icon">
      <svg className="block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 32 32">
        <g id="Icon">
          <path d={svgPaths.p314f200} id="Vector" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.99929" />
        </g>
      </svg>
    </div>
  );
}

function Container1() {
  return (
    <div className="bg-[#1a1a1a] box-border content-stretch flex items-center justify-center pl-[0.909px] pr-[0.923px] py-[0.909px] relative rounded-[16px] shrink-0 size-[63.992px]" data-name="Container">
      <div aria-hidden="true" className="absolute border-[0.909px] border-neutral-800 border-solid inset-0 pointer-events-none rounded-[16px]" />
      <Icon1 />
    </div>
  );
}

function Heading1() {
  return (
    <div className="h-[20px] relative shrink-0 w-full" data-name="Heading 3">
      <p className="absolute font-['Inter:Regular',sans-serif] font-normal leading-[20px] left-[68.56px] not-italic text-[14px] text-center text-nowrap text-white top-[0.82px] tracking-[-0.1504px] translate-x-[-50%] whitespace-pre">Workspace Panel</p>
    </div>
  );
}

function Paragraph() {
  return (
    <div className="content-stretch flex h-[15.994px] items-start relative shrink-0 w-full" data-name="Paragraph">
      <p className="font-['Inter:Regular',sans-serif] font-normal leading-[16px] not-italic relative shrink-0 text-[#888888] text-[12px] text-center text-nowrap whitespace-pre">Content will appear here</p>
    </div>
  );
}

function Container2() {
  return (
    <div className="absolute content-stretch flex flex-col gap-[10px] items-center left-[calc(50%+0.055px)] top-[calc(50%-0.818px)] translate-x-[-50%] translate-y-[-50%] w-[138.111px]" data-name="Container">
      <Container1 />
      <Heading1 />
      <Paragraph />
    </div>
  );
}

function Frame() {
  return (
    <div className="basis-0 grow min-h-px min-w-px relative shrink-0 w-full">
      <Container2 />
    </div>
  );
}

interface WorkspacePanelProps {
  title: string;
  onClose: () => void;
}

export default function WorkspacePanel({ title, onClose }: WorkspacePanelProps) {
  return (
    <div className="bg-[#0d0d0d] content-stretch flex flex-col items-center relative rounded-[16px] size-full" data-name="WorkspacePanel">
      <div aria-hidden="true" className="absolute border-[0px_0.909px_0px_0px] border-neutral-800 border-solid inset-0 pointer-events-none rounded-[16px]" />
      <Container title={title} onClose={onClose} />
      <Frame />
    </div>
  );
}