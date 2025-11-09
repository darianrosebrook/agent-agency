import svgPaths from "./svg-quupl4zjo1";

function TextArea() {
  return (
    <div className="box-border content-stretch flex gap-[8px] items-end pb-0 pt-[4px] px-0 relative shrink-0 w-full" data-name="Text Area">
      <p className="font-['Inter:Regular',sans-serif] font-normal leading-[24px] not-italic relative shrink-0 text-[#555555] text-[16px] text-nowrap tracking-[-0.3125px] whitespace-pre">What should we build?</p>
    </div>
  );
}

function Icon() {
  return (
    <div className="absolute inset-[0.39%_2.02%_-0.43%_-2.06%]" data-name="Icon">
      <svg className="block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 16 16">
        <g id="Icon">
          <path d="M3.33333 8H12.6667" id="Vector" stroke="var(--stroke-0, #99A1AF)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.66667" />
          <path d="M8 3.33333V12.6667" id="Vector_2" stroke="var(--stroke-0, #99A1AF)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.66667" />
        </g>
      </svg>
    </div>
  );
}

function Icon1() {
  return (
    <div className="relative shrink-0 size-[15.994px]" data-name="Icon">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border overflow-clip relative rounded-[inherit] size-[15.994px]">
        <Icon />
      </div>
    </div>
  );
}

function Button() {
  return (
    <div className="bg-[#1a1a1a] relative rounded-[8px] shrink-0 size-[32px]" data-name="Button">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border content-stretch flex items-center p-[8px] relative size-[32px]">
        <Icon1 />
      </div>
    </div>
  );
}

function Icon2() {
  return (
    <div className="relative shrink-0 size-[15.994px]" data-name="Icon">
      <svg className="block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 16 16">
        <g clipPath="url(#clip0_3_387)" id="Icon">
          <path d={svgPaths.p2e209400} id="Vector" stroke="var(--stroke-0, #99A1AF)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
          <path d={svgPaths.p2c300140} id="Vector_2" stroke="var(--stroke-0, #99A1AF)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
          <path d="M1.33286 7.99716H14.6615" id="Vector_3" stroke="var(--stroke-0, #99A1AF)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
        </g>
        <defs>
          <clipPath id="clip0_3_387">
            <rect fill="white" height="15.9943" width="15.9943" />
          </clipPath>
        </defs>
      </svg>
    </div>
  );
}

function Text() {
  return (
    <div className="h-[20px] relative shrink-0 w-[79.063px]" data-name="Text">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border h-[20px] relative w-[79.063px]">
        <p className="absolute font-['Inter:Regular',sans-serif] font-normal leading-[20px] left-0 not-italic text-[#99a1af] text-[14px] text-nowrap top-[0.82px] tracking-[-0.1504px] whitespace-pre">DeepSearch</p>
      </div>
    </div>
  );
}

function Button1() {
  return (
    <div className="bg-[#1a1a1a] h-[31.989px] relative rounded-[8px] shrink-0 w-[127.031px]" data-name="Button">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border content-stretch flex gap-[7.997px] h-[31.989px] items-center pl-[11.989px] pr-0 py-0 relative w-[127.031px]">
        <Icon2 />
        <Text />
      </div>
    </div>
  );
}

function Icon3() {
  return (
    <div className="relative shrink-0 size-[15.994px]" data-name="Icon">
      <svg className="block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 16 16">
        <g clipPath="url(#clip0_3_392)" id="Icon">
          <path d={svgPaths.p27072b00} id="Vector" stroke="var(--stroke-0, #99A1AF)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
          <path d="M5.99787 11.9957H9.99645" id="Vector_2" stroke="var(--stroke-0, #99A1AF)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
          <path d="M6.6643 14.6615H9.33002" id="Vector_3" stroke="var(--stroke-0, #99A1AF)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
        </g>
        <defs>
          <clipPath id="clip0_3_392">
            <rect fill="white" height="15.9943" width="15.9943" />
          </clipPath>
        </defs>
      </svg>
    </div>
  );
}

function Text1() {
  return (
    <div className="basis-0 grow h-[20px] min-h-px min-w-px relative shrink-0" data-name="Text">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border h-[20px] relative w-full">
        <p className="absolute font-['Inter:Regular',sans-serif] font-normal leading-[20px] left-0 not-italic text-[#99a1af] text-[14px] text-nowrap top-[0.82px] tracking-[-0.1504px] whitespace-pre">Think</p>
      </div>
    </div>
  );
}

function Button2() {
  return (
    <div className="bg-[#1a1a1a] h-[31.989px] relative rounded-[8px] shrink-0 w-[83.565px]" data-name="Button">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border content-stretch flex gap-[7.997px] h-[31.989px] items-center px-[11.989px] py-0 relative w-[83.565px]">
        <Icon3 />
        <Text1 />
      </div>
    </div>
  );
}

function Container() {
  return (
    <div className="h-[31.989px] relative shrink-0 w-[258.58px]" data-name="Container">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border content-stretch flex gap-[7.997px] h-[31.989px] items-center relative w-[258.58px]">
        <Button />
        <Button1 />
        <Button2 />
      </div>
    </div>
  );
}

function Button3() {
  return (
    <div className="relative shrink-0 size-[31.989px]" data-name="Button">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border size-[31.989px]" />
    </div>
  );
}

function Send() {
  return (
    <div className="absolute left-[-1.3px] size-[19.2px] top-[-1.64px]" data-name="Send">
      <svg className="block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 20 20">
        <g id="Send">
          <path d={svgPaths.p7df7e00} id="Vector" stroke="var(--stroke-0, #99A1AF)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.6" />
          <path d={svgPaths.p25491b40} id="Vector_2" stroke="var(--stroke-0, #99A1AF)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.6" />
        </g>
      </svg>
    </div>
  );
}

function SendIcon() {
  return (
    <div className="relative shrink-0 size-[16px]" data-name="SendIcon">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border relative size-[16px]">
        <Send />
      </div>
    </div>
  );
}

function Button4() {
  return (
    <div className="bg-[#1a1a1a] relative rounded-[8px] shrink-0 size-[32px]" data-name="Button">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border content-stretch flex items-center p-[8px] relative size-[32px]">
        <SendIcon />
      </div>
    </div>
  );
}

function Container1() {
  return (
    <div className="bg-[#0f0f0f] relative rounded-[12px] shrink-0 w-full" data-name="Container">
      <div aria-hidden="true" className="absolute border-[#1a1a1a] border-[0.909px] border-solid inset-0 pointer-events-none rounded-[12px]" />
      <div className="flex flex-row items-center size-full">
        <div className="box-border content-stretch flex items-center justify-between p-[4.909px] relative w-full">
          <Container />
          <Button3 />
          <Button4 />
        </div>
      </div>
    </div>
  );
}

export default function PromptBox() {
  return (
    <div className="bg-[#1a1a1a] relative rounded-[16px] size-full" data-name="Prompt box">
      <div aria-hidden="true" className="absolute border-[#1a1a1a] border-[0.909px] border-solid inset-0 pointer-events-none rounded-[16px]" />
      <div className="size-full">
        <div className="box-border content-stretch flex flex-col gap-[12px] items-start p-[8px] relative size-full">
          <TextArea />
          <Container1 />
        </div>
      </div>
    </div>
  );
}