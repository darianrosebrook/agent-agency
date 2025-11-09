import svgPaths from "./svg-687hlbd7g6";

function Heading() {
  return (
    <div className="h-[23.999px] relative shrink-0 w-[42.053px]" data-name="Heading 3">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border h-[23.999px] relative w-[42.053px]">
        <p className="absolute font-['Inter:Regular',sans-serif] font-normal leading-[24px] left-0 not-italic text-[16px] text-nowrap text-white top-[-0.73px] tracking-[-0.3125px] whitespace-pre">To Do</p>
      </div>
    </div>
  );
}

function Text() {
  return (
    <div className="h-[19.993px] relative shrink-0 w-[50.469px]" data-name="Text">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border h-[19.993px] relative w-[50.469px]">
        <p className="absolute font-['Inter:Regular',sans-serif] font-normal leading-[20px] left-0 not-italic text-[#888888] text-[14px] top-[0.36px] tracking-[-0.1504px] w-[51px]">3 Cards</p>
      </div>
    </div>
  );
}

function Container() {
  return (
    <div className="h-[23.999px] relative shrink-0 w-[100.518px]" data-name="Container">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border content-stretch flex gap-[7.997px] h-[23.999px] items-center relative w-[100.518px]">
        <Heading />
        <Text />
      </div>
    </div>
  );
}

function Icon() {
  return (
    <div className="relative shrink-0 size-[15.994px]" data-name="Icon">
      <svg className="block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 16 16">
        <g id="Icon">
          <path d={svgPaths.p65f2d00} id="Vector" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
          <path d={svgPaths.pb148ac0} id="Vector_2" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
          <path d={svgPaths.p19650e00} id="Vector_3" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
        </g>
      </svg>
    </div>
  );
}

function Button() {
  return (
    <div className="relative rounded-[8px] shrink-0 size-[23.999px]" data-name="Button">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border content-stretch flex items-center justify-center pl-0 pr-[0.007px] py-0 relative size-[23.999px]">
        <Icon />
      </div>
    </div>
  );
}

function Container1() {
  return (
    <div className="h-[23.999px] relative shrink-0 w-[639.41px]" data-name="Container">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border content-stretch flex h-[23.999px] items-center justify-between relative w-[639.41px]">
        <Container />
        <Button />
      </div>
    </div>
  );
}

function Icon1() {
  return (
    <div className="relative shrink-0 size-[15.994px]" data-name="Icon">
      <svg className="block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 16 16">
        <g id="Icon">
          <path d="M3.33215 7.99716H12.6622" id="Vector" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
          <path d="M7.99716 3.33215V12.6622" id="Vector_2" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
        </g>
      </svg>
    </div>
  );
}

function Button1({ status }: { status?: string }) {
  return (
    <button 
      className="h-[41.804px] relative rounded-[10px] shrink-0 w-[639.41px] hover:bg-[#1a1a1a] transition-colors cursor-pointer" 
      data-name="Button"
      data-add-task="true"
      data-status={status}
    >
      <div aria-hidden="true" className="absolute border-[0.909px] border-neutral-800 border-solid inset-0 pointer-events-none rounded-[10px]" />
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border content-stretch flex h-[41.804px] items-center justify-center pl-[0.909px] pr-[0.916px] py-[0.909px] relative w-[639.41px]">
        <Icon1 />
      </div>
    </button>
  );
}

function Icon2() {
  return (
    <div className="relative shrink-0 size-[15.994px]" data-name="Icon">
      <svg className="block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 16 16">
        <g clipPath="url(#clip0_16_2986)" id="Icon">
          <path d={svgPaths.p12f58300} id="Vector" stroke="var(--stroke-0, #FF9F43)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
          <path d="M7.99716 5.99787V8.66359" id="Vector_2" stroke="var(--stroke-0, #FF9F43)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
          <path d="M7.99716 11.3293H8.00382" id="Vector_3" stroke="var(--stroke-0, #FF9F43)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
        </g>
        <defs>
          <clipPath id="clip0_16_2986">
            <rect fill="white" height="15.9943" width="15.9943" />
          </clipPath>
        </defs>
      </svg>
    </div>
  );
}

function Text1() {
  return (
    <div className="h-[19.993px] relative shrink-0 w-[30.021px]" data-name="Text">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border h-[19.993px] relative w-[30.021px]">
        <p className="absolute font-['Inter:Regular',sans-serif] font-normal leading-[20px] left-0 not-italic text-[#ff9f43] text-[14px] text-nowrap top-[0.36px] tracking-[-0.1504px] whitespace-pre">High</p>
      </div>
    </div>
  );
}

function StatusTag() {
  return (
    <div className="absolute bg-[#3a2f1f] box-border content-stretch flex gap-[5.994px] h-[31.982px] items-center left-0 pl-[11.996px] pr-0 py-0 rounded-[1.5252e+07px] top-0 w-[76.001px]" data-name="StatusTag">
      <Icon2 />
      <Text1 />
    </div>
  );
}

function Text2() {
  return (
    <div className="h-[19.993px] relative shrink-0 w-[60.739px]" data-name="Text">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border h-[19.993px] relative w-[60.739px]">
        <p className="absolute font-['Inter:Regular',sans-serif] font-normal leading-[20px] left-0 not-italic text-[#888888] text-[14px] text-nowrap top-[0.36px] tracking-[-0.1504px] whitespace-pre">UI design</p>
      </div>
    </div>
  );
}

function StatusTag1() {
  return (
    <div className="absolute bg-neutral-800 box-border content-stretch flex h-[31.982px] items-center left-[84px] pl-[11.996px] pr-0 py-0 rounded-[1.5252e+07px] top-0 w-[84.73px]" data-name="StatusTag">
      <Text2 />
    </div>
  );
}

function Text3() {
  return (
    <div className="h-[19.993px] relative shrink-0 w-[69.261px]" data-name="Text">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border h-[19.993px] relative w-[69.261px]">
        <p className="absolute font-['Inter:Regular',sans-serif] font-normal leading-[20px] left-0 not-italic text-[#888888] text-[14px] text-nowrap top-[0.36px] tracking-[-0.1504px] whitespace-pre">Interaction</p>
      </div>
    </div>
  );
}

function StatusTag2() {
  return (
    <div className="absolute bg-neutral-800 box-border content-stretch flex h-[31.982px] items-center left-[176.73px] pl-[11.996px] pr-0 py-0 rounded-[1.5252e+07px] top-0 w-[93.253px]" data-name="StatusTag">
      <Text3 />
    </div>
  );
}

function Container2() {
  return (
    <div className="absolute h-[31.982px] left-0 top-0 w-[269.979px]" data-name="Container">
      <StatusTag />
      <StatusTag1 />
      <StatusTag2 />
    </div>
  );
}

function Icon3() {
  return (
    <div className="relative shrink-0 size-[15.994px]" data-name="Icon">
      <svg className="block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 16 16">
        <g id="Icon">
          <path d={svgPaths.p65f2d00} id="Vector" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
          <path d={svgPaths.pb148ac0} id="Vector_2" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
          <path d={svgPaths.p19650e00} id="Vector_3" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
        </g>
      </svg>
    </div>
  );
}

function Button2() {
  return (
    <div className="absolute content-stretch flex items-center justify-center left-[589.6px] rounded-[8px] size-[20px] top-[-4px]" data-name="Button">
      <Icon3 />
    </div>
  );
}

function Container3() {
  return (
    <div className="absolute h-[31.982px] left-[16.9px] top-[16.9px] w-[605.604px]" data-name="Container">
      <Container2 />
      <Button2 />
    </div>
  );
}

function Heading1() {
  return (
    <div className="absolute h-[23.999px] left-[16.9px] top-[60.88px] w-[605.604px]" data-name="Heading 4">
      <p className="absolute font-['Inter:Regular',sans-serif] font-normal leading-[24px] left-0 not-italic text-[16px] text-nowrap text-white top-[-0.73px] tracking-[-0.3125px] whitespace-pre">Implement drag-and-drop for task cards</p>
    </div>
  );
}

function Paragraph() {
  return (
    <div className="absolute h-[19.993px] left-[16.9px] overflow-clip top-[92.88px] w-[605.604px]" data-name="Paragraph">
      <p className="absolute font-['Inter:Regular',sans-serif] font-normal leading-[20px] left-0 not-italic text-[#888888] text-[14px] text-nowrap top-[0.36px] tracking-[-0.1504px] whitespace-pre">Enable users to drag-and-drop interaction for task cards using a smooth animation.</p>
    </div>
  );
}

function PrimitiveImg() {
  return (
    <div className="relative shrink-0 size-[20.362px]" data-name="Primitive.img">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border size-[20.362px]" />
    </div>
  );
}

function PrimitiveSpan() {
  return (
    <div className="absolute left-0 rounded-[1.5252e+07px] size-[23.999px] top-0" data-name="Primitive.span">
      <div className="box-border content-stretch flex items-start overflow-clip p-[1.818px] relative rounded-[inherit] size-[23.999px]">
        <PrimitiveImg />
      </div>
      <div aria-hidden="true" className="absolute border-[#1a1a1a] border-[1.818px] border-solid inset-0 pointer-events-none rounded-[1.5252e+07px]" />
    </div>
  );
}

function PrimitiveImg1() {
  return (
    <div className="relative shrink-0 size-[20.362px]" data-name="Primitive.img">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border size-[20.362px]" />
    </div>
  );
}

function PrimitiveSpan1() {
  return (
    <div className="absolute left-[16px] rounded-[1.5252e+07px] size-[23.999px] top-0" data-name="Primitive.span">
      <div className="box-border content-stretch flex items-start overflow-clip p-[1.818px] relative rounded-[inherit] size-[23.999px]">
        <PrimitiveImg1 />
      </div>
      <div aria-hidden="true" className="absolute border-[#1a1a1a] border-[1.818px] border-solid inset-0 pointer-events-none rounded-[1.5252e+07px]" />
    </div>
  );
}

function PrimitiveImg2() {
  return (
    <div className="relative shrink-0 size-[20.362px]" data-name="Primitive.img">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border size-[20.362px]" />
    </div>
  );
}

function PrimitiveSpan2() {
  return (
    <div className="absolute left-[32px] rounded-[1.5252e+07px] size-[23.999px] top-0" data-name="Primitive.span">
      <div className="box-border content-stretch flex items-start overflow-clip p-[1.818px] relative rounded-[inherit] size-[23.999px]">
        <PrimitiveImg2 />
      </div>
      <div aria-hidden="true" className="absolute border-[#1a1a1a] border-[1.818px] border-solid inset-0 pointer-events-none rounded-[1.5252e+07px]" />
    </div>
  );
}

function PrimitiveImg3() {
  return (
    <div className="relative shrink-0 size-[20.362px]" data-name="Primitive.img">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border size-[20.362px]" />
    </div>
  );
}

function PrimitiveSpan3() {
  return (
    <div className="absolute left-[48px] rounded-[1.5252e+07px] size-[23.999px] top-0" data-name="Primitive.span">
      <div className="box-border content-stretch flex items-start overflow-clip p-[1.818px] relative rounded-[inherit] size-[23.999px]">
        <PrimitiveImg3 />
      </div>
      <div aria-hidden="true" className="absolute border-[#1a1a1a] border-[1.818px] border-solid inset-0 pointer-events-none rounded-[1.5252e+07px]" />
    </div>
  );
}

function Container4() {
  return (
    <div className="h-[23.999px] relative shrink-0 w-[72.003px]" data-name="Container">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border h-[23.999px] relative w-[72.003px]">
        <PrimitiveSpan />
        <PrimitiveSpan1 />
        <PrimitiveSpan2 />
        <PrimitiveSpan3 />
      </div>
    </div>
  );
}

function Icon4() {
  return (
    <div className="relative shrink-0 size-[15.994px]" data-name="Icon">
      <svg className="block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 16 16">
        <g id="Icon">
          <path d="M3.33215 7.99716H12.6622" id="Vector" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
          <path d="M7.99715 3.33215V12.6622" id="Vector_2" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
        </g>
      </svg>
    </div>
  );
}

function Button3() {
  return (
    <div className="relative rounded-[8px] shrink-0 size-[23.999px]" data-name="Button">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border content-stretch flex items-center justify-center pl-0 pr-[0.007px] py-0 relative size-[23.999px]">
        <Icon4 />
      </div>
    </div>
  );
}

function Container5() {
  return (
    <div className="absolute content-stretch flex gap-[3.999px] h-[23.999px] items-center left-[16.9px] top-[128.86px] w-[605.604px]" data-name="Container">
      <Container4 />
      <Button3 />
    </div>
  );
}

function Icon5() {
  return (
    <div className="relative shrink-0 size-[13.999px]" data-name="Icon">
      <svg className="block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 14 14">
        <g clipPath="url(#clip0_16_2991)" id="Icon">
          <path d="M4.66619 1.16655V3.49964" id="Vector" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.16655" />
          <path d="M9.33239 1.16655V3.49964" id="Vector_2" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.16655" />
          <path d={svgPaths.p3c05b400} id="Vector_3" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.16655" />
          <path d="M1.74982 5.83274H12.2488" id="Vector_4" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.16655" />
        </g>
        <defs>
          <clipPath id="clip0_16_2991">
            <rect fill="white" height="13.9986" width="13.9986" />
          </clipPath>
        </defs>
      </svg>
    </div>
  );
}

function Text4() {
  return (
    <div className="basis-0 grow h-[19.993px] min-h-px min-w-px relative shrink-0" data-name="Text">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border h-[19.993px] relative w-full">
        <p className="absolute font-['Inter:Regular',sans-serif] font-normal leading-[20px] left-0 not-italic text-[#888888] text-[14px] text-nowrap top-[0.36px] tracking-[-0.1504px] whitespace-pre">July 2</p>
      </div>
    </div>
  );
}

function Container6() {
  return (
    <div className="h-[19.993px] relative shrink-0 w-[58.324px]" data-name="Container">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border content-stretch flex gap-[5.994px] h-[19.993px] items-center relative w-[58.324px]">
        <Icon5 />
        <Text4 />
      </div>
    </div>
  );
}

function Icon6() {
  return (
    <div className="relative shrink-0 size-[13.999px]" data-name="Icon">
      <svg className="block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 14 14">
        <g clipPath="url(#clip0_16_2980)" id="Icon">
          <path d={svgPaths.p29305080} id="Vector" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.16655" />
        </g>
        <defs>
          <clipPath id="clip0_16_2980">
            <rect fill="white" height="13.9986" width="13.9986" />
          </clipPath>
        </defs>
      </svg>
    </div>
  );
}

function Text5() {
  return (
    <div className="h-[19.993px] relative shrink-0 w-[17.067px]" data-name="Text">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border h-[19.993px] relative w-[17.067px]">
        <p className="absolute font-['Inter:Regular',sans-serif] font-normal leading-[20px] left-0 not-italic text-[#888888] text-[14px] text-nowrap top-[0.36px] tracking-[-0.1504px] whitespace-pre">26</p>
      </div>
    </div>
  );
}

function Container7() {
  return (
    <div className="h-[19.993px] relative shrink-0 w-[37.06px]" data-name="Container">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border content-stretch flex gap-[5.994px] h-[19.993px] items-center relative w-[37.06px]">
        <Icon6 />
        <Text5 />
      </div>
    </div>
  );
}

function Icon7() {
  return (
    <div className="relative shrink-0 size-[13.999px]" data-name="Icon">
      <svg className="block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 14 14">
        <g clipPath="url(#clip0_16_3017)" id="Icon">
          <path d={svgPaths.p2511aa00} id="Vector" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.16655" />
        </g>
        <defs>
          <clipPath id="clip0_16_3017">
            <rect fill="white" height="13.9986" width="13.9986" />
          </clipPath>
        </defs>
      </svg>
    </div>
  );
}

function Text6() {
  return (
    <div className="h-[19.993px] relative shrink-0 w-[8.303px]" data-name="Text">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border h-[19.993px] relative w-[8.303px]">
        <p className="absolute font-['Inter:Regular',sans-serif] font-normal leading-[20px] left-0 not-italic text-[#888888] text-[14px] text-nowrap top-[0.36px] tracking-[-0.1504px] whitespace-pre">2</p>
      </div>
    </div>
  );
}

function Container8() {
  return (
    <div className="h-[19.993px] relative shrink-0 w-[28.296px]" data-name="Container">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border content-stretch flex gap-[5.994px] h-[19.993px] items-center relative w-[28.296px]">
        <Icon7 />
        <Text6 />
      </div>
    </div>
  );
}

function Container9() {
  return (
    <div className="absolute content-stretch flex gap-[15.994px] h-[19.993px] items-center left-[16.9px] top-[168.86px] w-[605.604px]" data-name="Container">
      <Container6 />
      <Container7 />
      <Container8 />
    </div>
  );
}

function KanbanCard() {
  return (
    <div className="bg-[#1a1a1a] h-[205.753px] relative rounded-[10px] shrink-0 w-full" data-name="KanbanCard">
      <div aria-hidden="true" className="absolute border-[0.909px] border-neutral-800 border-solid inset-0 pointer-events-none rounded-[10px]" />
      <Container3 />
      <Heading1 />
      <Paragraph />
      <Container5 />
      <Container9 />
    </div>
  );
}

function Icon8() {
  return (
    <div className="relative shrink-0 size-[15.994px]" data-name="Icon">
      <svg className="block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 16 16">
        <g clipPath="url(#clip0_16_3007)" id="Icon">
          <path d={svgPaths.p24c0fe0} id="Vector" stroke="var(--stroke-0, #54A0FF)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
          <path d={svgPaths.p2d496d00} id="Vector_2" stroke="var(--stroke-0, #54A0FF)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
          <path d={svgPaths.p1b0f4080} id="Vector_3" stroke="var(--stroke-0, #54A0FF)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
          <path d={svgPaths.p21fcfc00} id="Vector_4" stroke="var(--stroke-0, #54A0FF)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
          <path d={svgPaths.p149b8d40} id="Vector_5" stroke="var(--stroke-0, #54A0FF)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
          <path d={svgPaths.p2320cf40} id="Vector_6" stroke="var(--stroke-0, #54A0FF)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
          <path d={svgPaths.pdc85680} id="Vector_7" stroke="var(--stroke-0, #54A0FF)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
          <path d={svgPaths.p1a30f300} id="Vector_8" stroke="var(--stroke-0, #54A0FF)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
        </g>
        <defs>
          <clipPath id="clip0_16_3007">
            <rect fill="white" height="15.9943" width="15.9943" />
          </clipPath>
        </defs>
      </svg>
    </div>
  );
}

function Text7() {
  return (
    <div className="h-[19.993px] relative shrink-0 w-[51.747px]" data-name="Text">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border h-[19.993px] relative w-[51.747px]">
        <p className="absolute font-['Inter:Regular',sans-serif] font-normal leading-[20px] left-0 not-italic text-[#54a0ff] text-[14px] text-nowrap top-[0.36px] tracking-[-0.1504px] whitespace-pre">Medium</p>
      </div>
    </div>
  );
}

function StatusTag3() {
  return (
    <div className="absolute bg-[#1f2d3a] box-border content-stretch flex gap-[5.994px] h-[31.982px] items-center left-0 pl-[11.996px] pr-0 py-0 rounded-[1.5252e+07px] top-0 w-[97.727px]" data-name="StatusTag">
      <Icon8 />
      <Text7 />
    </div>
  );
}

function Text8() {
  return (
    <div className="h-[19.993px] relative shrink-0 w-[19.46px]" data-name="Text">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border h-[19.993px] relative w-[19.46px]">
        <p className="absolute font-['Inter:Regular',sans-serif] font-normal leading-[20px] left-0 not-italic text-[#888888] text-[14px] text-nowrap top-[0.36px] tracking-[-0.1504px] whitespace-pre">QA</p>
      </div>
    </div>
  );
}

function StatusTag4() {
  return (
    <div className="absolute bg-neutral-800 box-border content-stretch flex h-[31.982px] items-center left-[105.72px] pl-[11.996px] pr-0 py-0 rounded-[1.5252e+07px] top-0 w-[43.452px]" data-name="StatusTag">
      <Text8 />
    </div>
  );
}

function Text9() {
  return (
    <div className="h-[19.993px] relative shrink-0 w-[33.658px]" data-name="Text">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border h-[19.993px] relative w-[33.658px]">
        <p className="absolute font-['Inter:Regular',sans-serif] font-normal leading-[20px] left-0 not-italic text-[#888888] text-[14px] text-nowrap top-[0.36px] tracking-[-0.1504px] whitespace-pre">Audit</p>
      </div>
    </div>
  );
}

function StatusTag5() {
  return (
    <div className="absolute bg-neutral-800 box-border content-stretch flex h-[31.982px] items-center left-[157.17px] pl-[11.996px] pr-0 py-0 rounded-[1.5252e+07px] top-0 w-[57.649px]" data-name="StatusTag">
      <Text9 />
    </div>
  );
}

function Container10() {
  return (
    <div className="absolute h-[31.982px] left-0 top-0 w-[214.822px]" data-name="Container">
      <StatusTag3 />
      <StatusTag4 />
      <StatusTag5 />
    </div>
  );
}

function Icon9() {
  return (
    <div className="relative shrink-0 size-[15.994px]" data-name="Icon">
      <svg className="block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 16 16">
        <g id="Icon">
          <path d={svgPaths.p65f2d00} id="Vector" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
          <path d={svgPaths.pb148ac0} id="Vector_2" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
          <path d={svgPaths.p19650e00} id="Vector_3" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
        </g>
      </svg>
    </div>
  );
}

function Button4() {
  return (
    <div className="absolute content-stretch flex items-center justify-center left-[589.6px] rounded-[8px] size-[20px] top-[-4px]" data-name="Button">
      <Icon9 />
    </div>
  );
}

function Container11() {
  return (
    <div className="absolute h-[31.982px] left-[16.9px] top-[16.9px] w-[605.604px]" data-name="Container">
      <Container10 />
      <Button4 />
    </div>
  );
}

function Heading2() {
  return (
    <div className="absolute h-[23.999px] left-[16.9px] top-[60.88px] w-[605.604px]" data-name="Heading 4">
      <p className="absolute font-['Inter:Regular',sans-serif] font-normal leading-[24px] left-0 not-italic text-[16px] text-nowrap text-white top-[-0.73px] tracking-[-0.3125px] whitespace-pre">Write unit tests for task filters</p>
    </div>
  );
}

function Paragraph1() {
  return (
    <div className="absolute h-[19.993px] left-[16.9px] overflow-clip top-[92.88px] w-[605.604px]" data-name="Paragraph">
      <p className="absolute font-['Inter:Regular',sans-serif] font-normal leading-[20px] left-0 not-italic text-[#888888] text-[14px] text-nowrap top-[0.36px] tracking-[-0.1504px] whitespace-pre">Ensure task filtering logic works correctly across various filter and sort combinations.</p>
    </div>
  );
}

function PrimitiveImg4() {
  return (
    <div className="relative shrink-0 size-[20.362px]" data-name="Primitive.img">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border size-[20.362px]" />
    </div>
  );
}

function PrimitiveSpan4() {
  return (
    <div className="absolute left-0 rounded-[1.5252e+07px] size-[23.999px] top-0" data-name="Primitive.span">
      <div className="box-border content-stretch flex items-start overflow-clip p-[1.818px] relative rounded-[inherit] size-[23.999px]">
        <PrimitiveImg4 />
      </div>
      <div aria-hidden="true" className="absolute border-[#1a1a1a] border-[1.818px] border-solid inset-0 pointer-events-none rounded-[1.5252e+07px]" />
    </div>
  );
}

function PrimitiveImg5() {
  return (
    <div className="relative shrink-0 size-[20.362px]" data-name="Primitive.img">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border size-[20.362px]" />
    </div>
  );
}

function PrimitiveSpan5() {
  return (
    <div className="absolute left-[16px] rounded-[1.5252e+07px] size-[23.999px] top-0" data-name="Primitive.span">
      <div className="box-border content-stretch flex items-start overflow-clip p-[1.818px] relative rounded-[inherit] size-[23.999px]">
        <PrimitiveImg5 />
      </div>
      <div aria-hidden="true" className="absolute border-[#1a1a1a] border-[1.818px] border-solid inset-0 pointer-events-none rounded-[1.5252e+07px]" />
    </div>
  );
}

function Container12() {
  return (
    <div className="h-[23.999px] relative shrink-0 w-[40px]" data-name="Container">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border h-[23.999px] relative w-[40px]">
        <PrimitiveSpan4 />
        <PrimitiveSpan5 />
      </div>
    </div>
  );
}

function Icon10() {
  return (
    <div className="relative shrink-0 size-[15.994px]" data-name="Icon">
      <svg className="block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 16 16">
        <g id="Icon">
          <path d="M3.33215 7.99716H12.6622" id="Vector" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
          <path d="M7.99716 3.33215V12.6622" id="Vector_2" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
        </g>
      </svg>
    </div>
  );
}

function Button5() {
  return (
    <div className="relative rounded-[8px] shrink-0 size-[23.999px]" data-name="Button">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border content-stretch flex items-center justify-center pl-0 pr-[0.007px] py-0 relative size-[23.999px]">
        <Icon10 />
      </div>
    </div>
  );
}

function Container13() {
  return (
    <div className="absolute content-stretch flex gap-[3.999px] h-[23.999px] items-center left-[16.9px] top-[128.86px] w-[605.604px]" data-name="Container">
      <Container12 />
      <Button5 />
    </div>
  );
}

function Icon11() {
  return (
    <div className="relative shrink-0 size-[13.999px]" data-name="Icon">
      <svg className="block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 14 14">
        <g clipPath="url(#clip0_16_2991)" id="Icon">
          <path d="M4.66619 1.16655V3.49964" id="Vector" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.16655" />
          <path d="M9.33239 1.16655V3.49964" id="Vector_2" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.16655" />
          <path d={svgPaths.p3c05b400} id="Vector_3" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.16655" />
          <path d="M1.74982 5.83274H12.2488" id="Vector_4" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.16655" />
        </g>
        <defs>
          <clipPath id="clip0_16_2991">
            <rect fill="white" height="13.9986" width="13.9986" />
          </clipPath>
        </defs>
      </svg>
    </div>
  );
}

function Text10() {
  return (
    <div className="basis-0 grow h-[19.993px] min-h-px min-w-px relative shrink-0" data-name="Text">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border h-[19.993px] relative w-full">
        <p className="absolute font-['Inter:Regular',sans-serif] font-normal leading-[20px] left-0 not-italic text-[#888888] text-[14px] text-nowrap top-[0.36px] tracking-[-0.1504px] whitespace-pre">July 5</p>
      </div>
    </div>
  );
}

function Container14() {
  return (
    <div className="h-[19.993px] relative shrink-0 w-[58.53px]" data-name="Container">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border content-stretch flex gap-[5.994px] h-[19.993px] items-center relative w-[58.53px]">
        <Icon11 />
        <Text10 />
      </div>
    </div>
  );
}

function Icon12() {
  return (
    <div className="relative shrink-0 size-[13.999px]" data-name="Icon">
      <svg className="block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 14 14">
        <g clipPath="url(#clip0_16_2980)" id="Icon">
          <path d={svgPaths.p29305080} id="Vector" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.16655" />
        </g>
        <defs>
          <clipPath id="clip0_16_2980">
            <rect fill="white" height="13.9986" width="13.9986" />
          </clipPath>
        </defs>
      </svg>
    </div>
  );
}

function Text11() {
  return (
    <div className="h-[19.993px] relative shrink-0 w-[14.645px]" data-name="Text">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border h-[19.993px] relative w-[14.645px]">
        <p className="absolute font-['Inter:Regular',sans-serif] font-normal leading-[20px] left-0 not-italic text-[#888888] text-[14px] text-nowrap top-[0.36px] tracking-[-0.1504px] whitespace-pre">12</p>
      </div>
    </div>
  );
}

function Container15() {
  return (
    <div className="h-[19.993px] relative shrink-0 w-[34.638px]" data-name="Container">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border content-stretch flex gap-[5.994px] h-[19.993px] items-center relative w-[34.638px]">
        <Icon12 />
        <Text11 />
      </div>
    </div>
  );
}

function Icon13() {
  return (
    <div className="relative shrink-0 size-[13.999px]" data-name="Icon">
      <svg className="block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 14 14">
        <g clipPath="url(#clip0_16_2977)" id="Icon">
          <path d={svgPaths.p3b670800} id="Vector" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.16655" />
        </g>
        <defs>
          <clipPath id="clip0_16_2977">
            <rect fill="white" height="13.9986" width="13.9986" />
          </clipPath>
        </defs>
      </svg>
    </div>
  );
}

function Text12() {
  return (
    <div className="h-[19.993px] relative shrink-0 w-[21.548px]" data-name="Text">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border h-[19.993px] relative w-[21.548px]">
        <p className="absolute font-['Inter:Regular',sans-serif] font-normal leading-[20px] left-0 not-italic text-[#888888] text-[14px] text-nowrap top-[0.36px] tracking-[-0.1504px] whitespace-pre">114</p>
      </div>
    </div>
  );
}

function Container16() {
  return (
    <div className="h-[19.993px] relative shrink-0 w-[41.541px]" data-name="Container">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border content-stretch flex gap-[5.994px] h-[19.993px] items-center relative w-[41.541px]">
        <Icon13 />
        <Text12 />
      </div>
    </div>
  );
}

function Container17() {
  return (
    <div className="absolute content-stretch flex gap-[15.994px] h-[19.993px] items-center left-[16.9px] top-[168.86px] w-[605.604px]" data-name="Container">
      <Container14 />
      <Container15 />
      <Container16 />
    </div>
  );
}

function KanbanCard1() {
  return (
    <div className="bg-[#1a1a1a] h-[205.753px] relative rounded-[10px] shrink-0 w-full" data-name="KanbanCard">
      <div aria-hidden="true" className="absolute border-[0.909px] border-neutral-800 border-solid inset-0 pointer-events-none rounded-[10px]" />
      <Container11 />
      <Heading2 />
      <Paragraph1 />
      <Container13 />
      <Container17 />
    </div>
  );
}

function Text13() {
  return (
    <div className="h-[19.993px] relative shrink-0 w-[25.171px]" data-name="Text">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border h-[19.993px] relative w-[25.171px]">
        <p className="absolute font-['Inter:Regular',sans-serif] font-normal leading-[20px] left-0 not-italic text-[#888888] text-[14px] text-nowrap top-[0.36px] tracking-[-0.1504px] whitespace-pre">Dev</p>
      </div>
    </div>
  );
}

function StatusTag6() {
  return (
    <div className="absolute bg-neutral-800 box-border content-stretch flex h-[31.982px] items-center left-0 pl-[11.996px] pr-0 py-0 rounded-[1.5252e+07px] top-0 w-[49.162px]" data-name="StatusTag">
      <Text13 />
    </div>
  );
}

function Text14() {
  return (
    <div className="h-[19.993px] relative shrink-0 w-[60.739px]" data-name="Text">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border h-[19.993px] relative w-[60.739px]">
        <p className="absolute font-['Inter:Regular',sans-serif] font-normal leading-[20px] left-0 not-italic text-[#888888] text-[14px] text-nowrap top-[0.36px] tracking-[-0.1504px] whitespace-pre">UI design</p>
      </div>
    </div>
  );
}

function StatusTag7() {
  return (
    <div className="absolute bg-neutral-800 box-border content-stretch flex h-[31.982px] items-center left-[57.16px] pl-[11.996px] pr-0 py-0 rounded-[1.5252e+07px] top-0 w-[84.73px]" data-name="StatusTag">
      <Text14 />
    </div>
  );
}

function Text15() {
  return (
    <div className="h-[19.993px] relative shrink-0 w-[67.813px]" data-name="Text">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border h-[19.993px] relative w-[67.813px]">
        <p className="absolute font-['Inter:Regular',sans-serif] font-normal leading-[20px] left-0 not-italic text-[#888888] text-[14px] text-nowrap top-[0.36px] tracking-[-0.1504px] whitespace-pre">Aesthetics</p>
      </div>
    </div>
  );
}

function StatusTag8() {
  return (
    <div className="absolute bg-neutral-800 box-border content-stretch flex h-[31.982px] items-center left-[149.89px] pl-[11.996px] pr-0 py-0 rounded-[1.5252e+07px] top-0 w-[91.804px]" data-name="StatusTag">
      <Text15 />
    </div>
  );
}

function Container18() {
  return (
    <div className="absolute h-[31.982px] left-0 top-0 w-[241.69px]" data-name="Container">
      <StatusTag6 />
      <StatusTag7 />
      <StatusTag8 />
    </div>
  );
}

function Icon14() {
  return (
    <div className="relative shrink-0 size-[15.994px]" data-name="Icon">
      <svg className="block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 16 16">
        <g id="Icon">
          <path d={svgPaths.p65f2d00} id="Vector" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
          <path d={svgPaths.pb148ac0} id="Vector_2" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
          <path d={svgPaths.p19650e00} id="Vector_3" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
        </g>
      </svg>
    </div>
  );
}

function Button6() {
  return (
    <div className="absolute content-stretch flex items-center justify-center left-[589.6px] rounded-[8px] size-[20px] top-[-4px]" data-name="Button">
      <Icon14 />
    </div>
  );
}

function Container19() {
  return (
    <div className="absolute h-[31.982px] left-[16.9px] top-[16.9px] w-[605.604px]" data-name="Container">
      <Container18 />
      <Button6 />
    </div>
  );
}

function Heading3() {
  return (
    <div className="absolute h-[23.999px] left-[16.9px] top-[60.88px] w-[605.604px]" data-name="Heading 4">
      <p className="absolute font-['Inter:Regular',sans-serif] font-normal leading-[24px] left-0 not-italic text-[16px] text-nowrap text-white top-[-0.73px] tracking-[-0.3125px] whitespace-pre">Add loading skeletons to task view</p>
    </div>
  );
}

function Paragraph2() {
  return (
    <div className="absolute h-[19.993px] left-[16.9px] overflow-clip top-[92.88px] w-[605.604px]" data-name="Paragraph">
      <p className="absolute font-['Inter:Regular',sans-serif] font-normal leading-[20px] left-0 not-italic text-[#888888] text-[14px] text-nowrap top-[0.36px] tracking-[-0.1504px] whitespace-pre">Improve perceived performance by adding skeleton screens.</p>
    </div>
  );
}

function PrimitiveImg6() {
  return (
    <div className="relative shrink-0 size-[20.362px]" data-name="Primitive.img">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border size-[20.362px]" />
    </div>
  );
}

function PrimitiveSpan6() {
  return (
    <div className="absolute left-0 rounded-[1.5252e+07px] size-[23.999px] top-0" data-name="Primitive.span">
      <div className="box-border content-stretch flex items-start overflow-clip p-[1.818px] relative rounded-[inherit] size-[23.999px]">
        <PrimitiveImg6 />
      </div>
      <div aria-hidden="true" className="absolute border-[#1a1a1a] border-[1.818px] border-solid inset-0 pointer-events-none rounded-[1.5252e+07px]" />
    </div>
  );
}

function Icon15() {
  return (
    <div className="relative shrink-0 size-[15.994px]" data-name="Icon">
      <svg className="block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 16 16">
        <g id="Icon">
          <path d="M3.33215 7.99716H12.6622" id="Vector" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
          <path d="M7.99716 3.33215V12.6622" id="Vector_2" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
        </g>
      </svg>
    </div>
  );
}

function Button7() {
  return (
    <div className="absolute box-border content-stretch flex items-center justify-center left-[28px] pl-0 pr-[0.007px] py-0 rounded-[8px] size-[23.999px] top-0" data-name="Button">
      <Icon15 />
    </div>
  );
}

function Container20() {
  return (
    <div className="absolute h-[23.999px] left-[16.9px] top-[128.86px] w-[605.604px]" data-name="Container">
      <PrimitiveSpan6 />
      <Button7 />
    </div>
  );
}

function Icon16() {
  return (
    <div className="relative shrink-0 size-[13.999px]" data-name="Icon">
      <svg className="block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 14 14">
        <g clipPath="url(#clip0_16_2991)" id="Icon">
          <path d="M4.66619 1.16655V3.49964" id="Vector" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.16655" />
          <path d="M9.33239 1.16655V3.49964" id="Vector_2" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.16655" />
          <path d={svgPaths.p3c05b400} id="Vector_3" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.16655" />
          <path d="M1.74982 5.83274H12.2488" id="Vector_4" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.16655" />
        </g>
        <defs>
          <clipPath id="clip0_16_2991">
            <rect fill="white" height="13.9986" width="13.9986" />
          </clipPath>
        </defs>
      </svg>
    </div>
  );
}

function Text16() {
  return (
    <div className="basis-0 grow h-[19.993px] min-h-px min-w-px relative shrink-0" data-name="Text">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border h-[19.993px] relative w-full">
        <p className="absolute font-['Inter:Regular',sans-serif] font-normal leading-[20px] left-0 not-italic text-[#888888] text-[14px] text-nowrap top-[0.36px] tracking-[-0.1504px] whitespace-pre">July 7</p>
      </div>
    </div>
  );
}

function Container21() {
  return (
    <div className="h-[19.993px] relative shrink-0 w-[57.848px]" data-name="Container">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border content-stretch flex gap-[5.994px] h-[19.993px] items-center relative w-[57.848px]">
        <Icon16 />
        <Text16 />
      </div>
    </div>
  );
}

function Icon17() {
  return (
    <div className="relative shrink-0 size-[13.999px]" data-name="Icon">
      <svg className="block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 14 14">
        <g clipPath="url(#clip0_16_2980)" id="Icon">
          <path d={svgPaths.p29305080} id="Vector" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.16655" />
        </g>
        <defs>
          <clipPath id="clip0_16_2980">
            <rect fill="white" height="13.9986" width="13.9986" />
          </clipPath>
        </defs>
      </svg>
    </div>
  );
}

function Text17() {
  return (
    <div className="h-[19.993px] relative shrink-0 w-[8.793px]" data-name="Text">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border h-[19.993px] relative w-[8.793px]">
        <p className="absolute font-['Inter:Regular',sans-serif] font-normal leading-[20px] left-0 not-italic text-[#888888] text-[14px] text-nowrap top-[0.36px] tracking-[-0.1504px] whitespace-pre">8</p>
      </div>
    </div>
  );
}

function Container22() {
  return (
    <div className="h-[19.993px] relative shrink-0 w-[28.785px]" data-name="Container">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border content-stretch flex gap-[5.994px] h-[19.993px] items-center relative w-[28.785px]">
        <Icon17 />
        <Text17 />
      </div>
    </div>
  );
}

function Icon18() {
  return (
    <div className="relative shrink-0 size-[13.999px]" data-name="Icon">
      <svg className="block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 14 14">
        <g clipPath="url(#clip0_16_2977)" id="Icon">
          <path d={svgPaths.p3b670800} id="Vector" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.16655" />
        </g>
        <defs>
          <clipPath id="clip0_16_2977">
            <rect fill="white" height="13.9986" width="13.9986" />
          </clipPath>
        </defs>
      </svg>
    </div>
  );
}

function Text18() {
  return (
    <div className="h-[19.993px] relative shrink-0 w-[8.629px]" data-name="Text">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border h-[19.993px] relative w-[8.629px]">
        <p className="absolute font-['Inter:Regular',sans-serif] font-normal leading-[20px] left-0 not-italic text-[#888888] text-[14px] text-nowrap top-[0.36px] tracking-[-0.1504px] whitespace-pre">3</p>
      </div>
    </div>
  );
}

function Container23() {
  return (
    <div className="h-[19.993px] relative shrink-0 w-[28.622px]" data-name="Container">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border content-stretch flex gap-[5.994px] h-[19.993px] items-center relative w-[28.622px]">
        <Icon18 />
        <Text18 />
      </div>
    </div>
  );
}

function Container24() {
  return (
    <div className="absolute content-stretch flex gap-[15.994px] h-[19.993px] items-center left-[16.9px] top-[168.86px] w-[605.604px]" data-name="Container">
      <Container21 />
      <Container22 />
      <Container23 />
    </div>
  );
}

function KanbanCard2() {
  return (
    <div className="bg-[#1a1a1a] h-[205.753px] relative rounded-[10px] shrink-0 w-full" data-name="KanbanCard">
      <div aria-hidden="true" className="absolute border-[0.909px] border-neutral-800 border-solid inset-0 pointer-events-none rounded-[10px]" />
      <Container19 />
      <Heading3 />
      <Paragraph2 />
      <Container20 />
      <Container24 />
    </div>
  );
}

function Container25() {
  return (
    <div className="h-[641.25px] relative shrink-0 w-[639.41px]" data-name="Container">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border content-stretch flex flex-col gap-[11.996px] h-[641.25px] items-start relative w-[639.41px]">
        <KanbanCard />
        <KanbanCard1 />
        <KanbanCard2 />
      </div>
    </div>
  );
}

function KanbanColumn() {
  return (
    <div className="absolute content-stretch flex flex-col gap-[15.994px] h-[735.043px] items-start left-0 top-0 w-[639.41px]" data-name="KanbanColumn">
      <Container1 />
      <Button1 status="todo" />
      <Container25 />
    </div>
  );
}

function Heading4() {
  return (
    <div className="h-[23.999px] relative shrink-0 w-[81.506px]" data-name="Heading 3">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border h-[23.999px] relative w-[81.506px]">
        <p className="absolute font-['Inter:Regular',sans-serif] font-normal leading-[24px] left-0 not-italic text-[16px] text-nowrap text-white top-[-0.73px] tracking-[-0.3125px] whitespace-pre">In Progress</p>
      </div>
    </div>
  );
}

function Text19() {
  return (
    <div className="h-[19.993px] relative shrink-0 w-[48.189px]" data-name="Text">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border h-[19.993px] relative w-[48.189px]">
        <p className="absolute font-['Inter:Regular',sans-serif] font-normal leading-[20px] left-0 not-italic text-[#888888] text-[14px] top-[0.36px] tracking-[-0.1504px] w-[49px]">1 Cards</p>
      </div>
    </div>
  );
}

function Container26() {
  return (
    <div className="h-[23.999px] relative shrink-0 w-[137.692px]" data-name="Container">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border content-stretch flex gap-[7.997px] h-[23.999px] items-center relative w-[137.692px]">
        <Heading4 />
        <Text19 />
      </div>
    </div>
  );
}

function Icon19() {
  return (
    <div className="relative shrink-0 size-[15.994px]" data-name="Icon">
      <svg className="block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 16 16">
        <g id="Icon">
          <path d={svgPaths.p65f2d00} id="Vector" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
          <path d={svgPaths.pb148ac0} id="Vector_2" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
          <path d={svgPaths.p19650e00} id="Vector_3" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
        </g>
      </svg>
    </div>
  );
}

function Button8() {
  return (
    <div className="relative rounded-[8px] shrink-0 size-[23.999px]" data-name="Button">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border content-stretch flex items-center justify-center pl-0 pr-[0.007px] py-0 relative size-[23.999px]">
        <Icon19 />
      </div>
    </div>
  );
}

function Container27() {
  return (
    <div className="h-[23.999px] relative shrink-0 w-[639.418px]" data-name="Container">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border content-stretch flex h-[23.999px] items-center justify-between relative w-[639.418px]">
        <Container26 />
        <Button8 />
      </div>
    </div>
  );
}

function Icon20() {
  return (
    <div className="relative shrink-0 size-[15.994px]" data-name="Icon">
      <svg className="block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 16 16">
        <g id="Icon">
          <path d="M3.33215 7.99716H12.6622" id="Vector" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
          <path d="M7.99716 3.33215V12.6622" id="Vector_2" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
        </g>
      </svg>
    </div>
  );
}

function Button9() {
  return (
    <div className="h-[41.804px] relative rounded-[10px] shrink-0 w-[639.418px]" data-name="Button">
      <div aria-hidden="true" className="absolute border-[0.909px] border-neutral-800 border-solid inset-0 pointer-events-none rounded-[10px]" />
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border content-stretch flex h-[41.804px] items-center justify-center p-[0.909px] relative w-[639.418px]">
        <Icon20 />
      </div>
    </div>
  );
}

function Icon21() {
  return (
    <div className="relative shrink-0 size-[15.994px]" data-name="Icon">
      <svg className="block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 16 16">
        <g clipPath="url(#clip0_16_2967)" id="Icon">
          <path d={svgPaths.p12263f80} id="Vector" stroke="var(--stroke-0, #54A0FF)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
          <path d={svgPaths.p1f683700} id="Vector_2" stroke="var(--stroke-0, #54A0FF)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
          <path d={svgPaths.p1b0f4080} id="Vector_3" stroke="var(--stroke-0, #54A0FF)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
          <path d={svgPaths.p23642eb0} id="Vector_4" stroke="var(--stroke-0, #54A0FF)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
          <path d={svgPaths.pe8d1500} id="Vector_5" stroke="var(--stroke-0, #54A0FF)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
          <path d={svgPaths.p2320cf40} id="Vector_6" stroke="var(--stroke-0, #54A0FF)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
          <path d={svgPaths.pdc85680} id="Vector_7" stroke="var(--stroke-0, #54A0FF)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
          <path d={svgPaths.p152bbea0} id="Vector_8" stroke="var(--stroke-0, #54A0FF)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
        </g>
        <defs>
          <clipPath id="clip0_16_2967">
            <rect fill="white" height="15.9943" width="15.9943" />
          </clipPath>
        </defs>
      </svg>
    </div>
  );
}

function Text20() {
  return (
    <div className="h-[19.993px] relative shrink-0 w-[51.747px]" data-name="Text">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border h-[19.993px] relative w-[51.747px]">
        <p className="absolute font-['Inter:Regular',sans-serif] font-normal leading-[20px] left-0 not-italic text-[#54a0ff] text-[14px] text-nowrap top-[0.36px] tracking-[-0.1504px] whitespace-pre">Medium</p>
      </div>
    </div>
  );
}

function StatusTag9() {
  return (
    <div className="absolute bg-[#1f2d3a] box-border content-stretch flex gap-[5.994px] h-[31.982px] items-center left-0 pl-[11.996px] pr-0 py-0 rounded-[1.5252e+07px] top-0 w-[97.727px]" data-name="StatusTag">
      <Icon21 />
      <Text20 />
    </div>
  );
}

function Text21() {
  return (
    <div className="h-[19.993px] relative shrink-0 w-[60.739px]" data-name="Text">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border h-[19.993px] relative w-[60.739px]">
        <p className="absolute font-['Inter:Regular',sans-serif] font-normal leading-[20px] left-0 not-italic text-[#888888] text-[14px] text-nowrap top-[0.36px] tracking-[-0.1504px] whitespace-pre">UI design</p>
      </div>
    </div>
  );
}

function StatusTag10() {
  return (
    <div className="absolute bg-neutral-800 box-border content-stretch flex h-[31.982px] items-center left-[105.72px] pl-[11.996px] pr-0 py-0 rounded-[1.5252e+07px] top-0 w-[84.73px]" data-name="StatusTag">
      <Text21 />
    </div>
  );
}

function Text22() {
  return (
    <div className="h-[19.993px] relative shrink-0 w-[69.261px]" data-name="Text">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border h-[19.993px] relative w-[69.261px]">
        <p className="absolute font-['Inter:Regular',sans-serif] font-normal leading-[20px] left-0 not-italic text-[#888888] text-[14px] text-nowrap top-[0.36px] tracking-[-0.1504px] whitespace-pre">Interaction</p>
      </div>
    </div>
  );
}

function StatusTag11() {
  return (
    <div className="absolute bg-neutral-800 box-border content-stretch flex h-[31.982px] items-center left-[198.45px] pl-[11.996px] pr-0 py-0 rounded-[1.5252e+07px] top-0 w-[93.253px]" data-name="StatusTag">
      <Text22 />
    </div>
  );
}

function Container28() {
  return (
    <div className="absolute h-[31.982px] left-0 top-0 w-[291.705px]" data-name="Container">
      <StatusTag9 />
      <StatusTag10 />
      <StatusTag11 />
    </div>
  );
}

function Icon22() {
  return (
    <div className="relative shrink-0 size-[15.994px]" data-name="Icon">
      <svg className="block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 16 16">
        <g id="Icon">
          <path d={svgPaths.p65f2d00} id="Vector" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
          <path d={svgPaths.pb148ac0} id="Vector_2" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
          <path d={svgPaths.p19650e00} id="Vector_3" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
        </g>
      </svg>
    </div>
  );
}

function Button10() {
  return (
    <div className="absolute content-stretch flex items-center justify-center left-[589.61px] rounded-[8px] size-[20px] top-[-4px]" data-name="Button">
      <Icon22 />
    </div>
  );
}

function Container29() {
  return (
    <div className="absolute h-[31.982px] left-[16.9px] top-[16.9px] w-[605.611px]" data-name="Container">
      <Container28 />
      <Button10 />
    </div>
  );
}

function Heading5() {
  return (
    <div className="absolute h-[23.999px] left-[16.9px] top-[60.88px] w-[605.611px]" data-name="Heading 4">
      <p className="absolute font-['Inter:Regular',sans-serif] font-normal leading-[24px] left-0 not-italic text-[16px] text-nowrap text-white top-[-0.73px] tracking-[-0.3125px] whitespace-pre">Build column reorder functionality</p>
    </div>
  );
}

function Paragraph3() {
  return (
    <div className="absolute h-[19.993px] left-[16.9px] overflow-clip top-[92.88px] w-[605.611px]" data-name="Paragraph">
      <p className="absolute font-['Inter:Regular',sans-serif] font-normal leading-[20px] left-0 not-italic text-[#888888] text-[14px] text-nowrap top-[0.36px] tracking-[-0.1504px] whitespace-pre">Allow users to drag columns to rearrange kanban columns their own way.</p>
    </div>
  );
}

function PrimitiveImg7() {
  return (
    <div className="relative shrink-0 size-[20.362px]" data-name="Primitive.img">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border size-[20.362px]" />
    </div>
  );
}

function PrimitiveSpan7() {
  return (
    <div className="absolute left-0 rounded-[1.5252e+07px] size-[23.999px] top-0" data-name="Primitive.span">
      <div className="box-border content-stretch flex items-start overflow-clip p-[1.818px] relative rounded-[inherit] size-[23.999px]">
        <PrimitiveImg7 />
      </div>
      <div aria-hidden="true" className="absolute border-[#1a1a1a] border-[1.818px] border-solid inset-0 pointer-events-none rounded-[1.5252e+07px]" />
    </div>
  );
}

function PrimitiveImg8() {
  return (
    <div className="relative shrink-0 size-[20.362px]" data-name="Primitive.img">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border size-[20.362px]" />
    </div>
  );
}

function PrimitiveSpan8() {
  return (
    <div className="absolute left-[16px] rounded-[1.5252e+07px] size-[23.999px] top-0" data-name="Primitive.span">
      <div className="box-border content-stretch flex items-start overflow-clip p-[1.818px] relative rounded-[inherit] size-[23.999px]">
        <PrimitiveImg8 />
      </div>
      <div aria-hidden="true" className="absolute border-[#1a1a1a] border-[1.818px] border-solid inset-0 pointer-events-none rounded-[1.5252e+07px]" />
    </div>
  );
}

function Container30() {
  return (
    <div className="h-[23.999px] relative shrink-0 w-[40px]" data-name="Container">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border h-[23.999px] relative w-[40px]">
        <PrimitiveSpan7 />
        <PrimitiveSpan8 />
      </div>
    </div>
  );
}

function Icon23() {
  return (
    <div className="relative shrink-0 size-[15.994px]" data-name="Icon">
      <svg className="block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 16 16">
        <g id="Icon">
          <path d="M3.33215 7.99716H12.6622" id="Vector" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
          <path d="M7.99716 3.33215V12.6622" id="Vector_2" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
        </g>
      </svg>
    </div>
  );
}

function Button11() {
  return (
    <div className="relative rounded-[8px] shrink-0 size-[23.999px]" data-name="Button">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border content-stretch flex items-center justify-center pl-0 pr-[0.007px] py-0 relative size-[23.999px]">
        <Icon23 />
      </div>
    </div>
  );
}

function Container31() {
  return (
    <div className="absolute content-stretch flex gap-[3.999px] h-[23.999px] items-center left-[16.9px] top-[128.86px] w-[605.611px]" data-name="Container">
      <Container30 />
      <Button11 />
    </div>
  );
}

function Icon24() {
  return (
    <div className="relative shrink-0 size-[13.999px]" data-name="Icon">
      <svg className="block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 14 14">
        <g clipPath="url(#clip0_16_2991)" id="Icon">
          <path d="M4.66619 1.16655V3.49964" id="Vector" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.16655" />
          <path d="M9.33239 1.16655V3.49964" id="Vector_2" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.16655" />
          <path d={svgPaths.p3c05b400} id="Vector_3" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.16655" />
          <path d="M1.74982 5.83274H12.2488" id="Vector_4" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.16655" />
        </g>
        <defs>
          <clipPath id="clip0_16_2991">
            <rect fill="white" height="13.9986" width="13.9986" />
          </clipPath>
        </defs>
      </svg>
    </div>
  );
}

function Text23() {
  return (
    <div className="h-[19.993px] relative shrink-0 w-[38.331px]" data-name="Text">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border h-[19.993px] relative w-[38.331px]">
        <p className="absolute font-['Inter:Regular',sans-serif] font-normal leading-[20px] left-0 not-italic text-[#888888] text-[14px] text-nowrap top-[0.36px] tracking-[-0.1504px] whitespace-pre">July 2</p>
      </div>
    </div>
  );
}

function Container32() {
  return (
    <div className="h-[19.993px] relative shrink-0 w-[58.324px]" data-name="Container">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border content-stretch flex gap-[5.994px] h-[19.993px] items-center relative w-[58.324px]">
        <Icon24 />
        <Text23 />
      </div>
    </div>
  );
}

function Icon25() {
  return (
    <div className="relative shrink-0 size-[13.999px]" data-name="Icon">
      <svg className="block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 14 14">
        <g clipPath="url(#clip0_16_2980)" id="Icon">
          <path d={svgPaths.p29305080} id="Vector" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.16655" />
        </g>
        <defs>
          <clipPath id="clip0_16_2980">
            <rect fill="white" height="13.9986" width="13.9986" />
          </clipPath>
        </defs>
      </svg>
    </div>
  );
}

function Text24() {
  return (
    <div className="h-[19.993px] relative shrink-0 w-[16.804px]" data-name="Text">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border h-[19.993px] relative w-[16.804px]">
        <p className="absolute font-['Inter:Regular',sans-serif] font-normal leading-[20px] left-0 not-italic text-[#888888] text-[14px] text-nowrap top-[0.36px] tracking-[-0.1504px] whitespace-pre">25</p>
      </div>
    </div>
  );
}

function Container33() {
  return (
    <div className="h-[19.993px] relative shrink-0 w-[36.797px]" data-name="Container">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border content-stretch flex gap-[5.994px] h-[19.993px] items-center relative w-[36.797px]">
        <Icon25 />
        <Text24 />
      </div>
    </div>
  );
}

function Icon26() {
  return (
    <div className="relative shrink-0 size-[13.999px]" data-name="Icon">
      <svg className="block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 14 14">
        <g clipPath="url(#clip0_16_2955)" id="Icon">
          <path d={svgPaths.p25e4e300} id="Vector" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.16655" />
        </g>
        <defs>
          <clipPath id="clip0_16_2955">
            <rect fill="white" height="13.9986" width="13.9986" />
          </clipPath>
        </defs>
      </svg>
    </div>
  );
}

function Text25() {
  return (
    <div className="h-[19.993px] relative shrink-0 w-[8.303px]" data-name="Text">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border h-[19.993px] relative w-[8.303px]">
        <p className="absolute font-['Inter:Regular',sans-serif] font-normal leading-[20px] left-0 not-italic text-[#888888] text-[14px] text-nowrap top-[0.36px] tracking-[-0.1504px] whitespace-pre">2</p>
      </div>
    </div>
  );
}

function Container34() {
  return (
    <div className="h-[19.993px] relative shrink-0 w-[28.296px]" data-name="Container">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border content-stretch flex gap-[5.994px] h-[19.993px] items-center relative w-[28.296px]">
        <Icon26 />
        <Text25 />
      </div>
    </div>
  );
}

function Container35() {
  return (
    <div className="absolute content-stretch flex gap-[15.994px] h-[19.993px] items-center left-[16.9px] top-[168.86px] w-[605.611px]" data-name="Container">
      <Container32 />
      <Container33 />
      <Container34 />
    </div>
  );
}

function KanbanCard3() {
  return (
    <div className="bg-[#1a1a1a] h-[205.753px] relative rounded-[10px] shrink-0 w-[639.418px]" data-name="KanbanCard">
      <div aria-hidden="true" className="absolute border-[0.909px] border-neutral-800 border-solid inset-0 pointer-events-none rounded-[10px]" />
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border h-[205.753px] relative w-[639.418px]">
        <Container29 />
        <Heading5 />
        <Paragraph3 />
        <Container31 />
        <Container35 />
      </div>
    </div>
  );
}

function KanbanColumn1() {
  return (
    <div className="absolute content-stretch flex flex-col gap-[15.994px] h-[735.043px] items-start left-[655.4px] top-0 w-[639.418px]" data-name="KanbanColumn">
      <Container27 />
      <Button9 />
      <KanbanCard3 />
    </div>
  );
}

function Heading6() {
  return (
    <div className="h-[23.999px] relative shrink-0 w-[68.665px]" data-name="Heading 3">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border h-[23.999px] relative w-[68.665px]">
        <p className="absolute font-['Inter:Regular',sans-serif] font-normal leading-[24px] left-0 not-italic text-[16px] text-nowrap text-white top-[-0.73px] tracking-[-0.3125px] whitespace-pre">In Review</p>
      </div>
    </div>
  );
}

function Text26() {
  return (
    <div className="h-[19.993px] relative shrink-0 w-[50.142px]" data-name="Text">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border h-[19.993px] relative w-[50.142px]">
        <p className="absolute font-['Inter:Regular',sans-serif] font-normal leading-[20px] left-0 not-italic text-[#888888] text-[14px] top-[0.36px] tracking-[-0.1504px] w-[51px]">2 Cards</p>
      </div>
    </div>
  );
}

function Container36() {
  return (
    <div className="h-[23.999px] relative shrink-0 w-[126.804px]" data-name="Container">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border content-stretch flex gap-[7.997px] h-[23.999px] items-center relative w-[126.804px]">
        <Heading6 />
        <Text26 />
      </div>
    </div>
  );
}

function Icon27() {
  return (
    <div className="relative shrink-0 size-[15.994px]" data-name="Icon">
      <svg className="block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 16 16">
        <g id="Icon">
          <path d={svgPaths.p65f2d00} id="Vector" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
          <path d={svgPaths.pb148ac0} id="Vector_2" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
          <path d={svgPaths.p19650e00} id="Vector_3" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
        </g>
      </svg>
    </div>
  );
}

function Button12() {
  return (
    <div className="relative rounded-[8px] shrink-0 size-[23.999px]" data-name="Button">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border content-stretch flex items-center justify-center pl-0 pr-[0.007px] py-0 relative size-[23.999px]">
        <Icon27 />
      </div>
    </div>
  );
}

function Container37() {
  return (
    <div className="h-[23.999px] relative shrink-0 w-[639.41px]" data-name="Container">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border content-stretch flex h-[23.999px] items-center justify-between relative w-[639.41px]">
        <Container36 />
        <Button12 />
      </div>
    </div>
  );
}

function Icon28() {
  return (
    <div className="relative shrink-0 size-[15.994px]" data-name="Icon">
      <svg className="block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 16 16">
        <g id="Icon">
          <path d="M3.33215 7.99716H12.6622" id="Vector" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
          <path d="M7.99716 3.33215V12.6622" id="Vector_2" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
        </g>
      </svg>
    </div>
  );
}

function Button13({ status }: { status?: string }) {
  return (
    <button 
      className="h-[41.804px] relative rounded-[10px] shrink-0 w-[639.41px] hover:bg-[#1a1a1a] transition-colors cursor-pointer" 
      data-name="Button"
      data-add-task="true"
      data-status={status}
    >
      <div aria-hidden="true" className="absolute border-[0.909px] border-neutral-800 border-solid inset-0 pointer-events-none rounded-[10px]" />
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border content-stretch flex h-[41.804px] items-center justify-center pl-[0.909px] pr-[0.916px] py-[0.909px] relative w-[639.41px]">
        <Icon28 />
      </div>
    </button>
  );
}

function Text27() {
  return (
    <div className="h-[19.993px] relative shrink-0 w-[25.171px]" data-name="Text">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border h-[19.993px] relative w-[25.171px]">
        <p className="absolute font-['Inter:Regular',sans-serif] font-normal leading-[20px] left-0 not-italic text-[#888888] text-[14px] text-nowrap top-[0.36px] tracking-[-0.1504px] whitespace-pre">Dev</p>
      </div>
    </div>
  );
}

function StatusTag12() {
  return (
    <div className="absolute bg-neutral-800 box-border content-stretch flex h-[31.982px] items-center left-0 pl-[11.996px] pr-0 py-0 rounded-[1.5252e+07px] top-0 w-[49.162px]" data-name="StatusTag">
      <Text27 />
    </div>
  );
}

function Text28() {
  return (
    <div className="h-[19.993px] relative shrink-0 w-[96.385px]" data-name="Text">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border h-[19.993px] relative w-[96.385px]">
        <p className="absolute font-['Inter:Regular',sans-serif] font-normal leading-[20px] left-0 not-italic text-[#888888] text-[14px] text-nowrap top-[0.36px] tracking-[-0.1504px] whitespace-pre">Design System</p>
      </div>
    </div>
  );
}

function StatusTag13() {
  return (
    <div className="absolute bg-neutral-800 box-border content-stretch flex h-[31.982px] items-center left-[57.16px] pl-[11.996px] pr-0 py-0 rounded-[1.5252e+07px] top-0 w-[120.376px]" data-name="StatusTag">
      <Text28 />
    </div>
  );
}

function Text29() {
  return (
    <div className="h-[19.993px] relative shrink-0 w-[33.004px]" data-name="Text">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border h-[19.993px] relative w-[33.004px]">
        <p className="absolute font-['Inter:Regular',sans-serif] font-normal leading-[20px] left-0 not-italic text-[#888888] text-[14px] text-nowrap top-[0.36px] tracking-[-0.1504px] whitespace-pre">Docs</p>
      </div>
    </div>
  );
}

function StatusTag14() {
  return (
    <div className="absolute bg-neutral-800 box-border content-stretch flex h-[31.982px] items-center left-[185.53px] pl-[11.996px] pr-0 py-0 rounded-[1.5252e+07px] top-0 w-[56.996px]" data-name="StatusTag">
      <Text29 />
    </div>
  );
}

function Container38() {
  return (
    <div className="absolute h-[31.982px] left-0 top-0 w-[242.528px]" data-name="Container">
      <StatusTag12 />
      <StatusTag13 />
      <StatusTag14 />
    </div>
  );
}

function Icon29() {
  return (
    <div className="relative shrink-0 size-[15.994px]" data-name="Icon">
      <svg className="block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 16 16">
        <g id="Icon">
          <path d={svgPaths.p65f2d00} id="Vector" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
          <path d={svgPaths.pb148ac0} id="Vector_2" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
          <path d={svgPaths.p19650e00} id="Vector_3" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
        </g>
      </svg>
    </div>
  );
}

function Button14() {
  return (
    <div className="absolute content-stretch flex items-center justify-center left-[589.6px] rounded-[8px] size-[20px] top-[-4px]" data-name="Button">
      <Icon29 />
    </div>
  );
}

function Container39() {
  return (
    <div className="absolute h-[31.982px] left-[16.9px] top-[16.9px] w-[605.604px]" data-name="Container">
      <Container38 />
      <Button14 />
    </div>
  );
}

function Heading7() {
  return (
    <div className="absolute h-[23.999px] left-[16.9px] top-[60.88px] w-[605.604px]" data-name="Heading 4">
      <p className="absolute font-['Inter:Regular',sans-serif] font-normal leading-[24px] left-0 not-italic text-[16px] text-nowrap text-white top-[-0.73px] tracking-[-0.3125px] whitespace-pre">Refactor task card component for modularity</p>
    </div>
  );
}

function Paragraph4() {
  return (
    <div className="absolute h-[39.986px] left-[16.9px] overflow-clip top-[92.88px] w-[605.604px]" data-name="Paragraph">
      <p className="absolute font-['Inter:Regular',sans-serif] font-normal leading-[20px] left-0 not-italic text-[#888888] text-[14px] top-[0.36px] tracking-[-0.1504px] w-[586px]">Break down the task card into smaller, reusable components to improve maintainability and scalability.</p>
    </div>
  );
}

function PrimitiveImg9() {
  return (
    <div className="relative shrink-0 size-[20.362px]" data-name="Primitive.img">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border size-[20.362px]" />
    </div>
  );
}

function PrimitiveSpan9() {
  return (
    <div className="absolute left-0 rounded-[1.5252e+07px] size-[23.999px] top-0" data-name="Primitive.span">
      <div className="box-border content-stretch flex items-start overflow-clip p-[1.818px] relative rounded-[inherit] size-[23.999px]">
        <PrimitiveImg9 />
      </div>
      <div aria-hidden="true" className="absolute border-[#1a1a1a] border-[1.818px] border-solid inset-0 pointer-events-none rounded-[1.5252e+07px]" />
    </div>
  );
}

function Icon30() {
  return (
    <div className="relative shrink-0 size-[15.994px]" data-name="Icon">
      <svg className="block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 16 16">
        <g id="Icon">
          <path d="M3.33215 7.99716H12.6622" id="Vector" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
          <path d="M7.99716 3.33215V12.6622" id="Vector_2" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
        </g>
      </svg>
    </div>
  );
}

function Button15() {
  return (
    <div className="absolute box-border content-stretch flex items-center justify-center left-[28px] pl-0 pr-[0.007px] py-0 rounded-[8px] size-[23.999px] top-0" data-name="Button">
      <Icon30 />
    </div>
  );
}

function Container40() {
  return (
    <div className="absolute h-[23.999px] left-[16.9px] top-[148.86px] w-[605.604px]" data-name="Container">
      <PrimitiveSpan9 />
      <Button15 />
    </div>
  );
}

function Icon31() {
  return (
    <div className="relative shrink-0 size-[13.999px]" data-name="Icon">
      <svg className="block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 14 14">
        <g clipPath="url(#clip0_16_2991)" id="Icon">
          <path d="M4.66619 1.16655V3.49964" id="Vector" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.16655" />
          <path d="M9.33239 1.16655V3.49964" id="Vector_2" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.16655" />
          <path d={svgPaths.p3c05b400} id="Vector_3" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.16655" />
          <path d="M1.74982 5.83274H12.2488" id="Vector_4" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.16655" />
        </g>
        <defs>
          <clipPath id="clip0_16_2991">
            <rect fill="white" height="13.9986" width="13.9986" />
          </clipPath>
        </defs>
      </svg>
    </div>
  );
}

function Text30() {
  return (
    <div className="h-[19.993px] relative shrink-0 w-[38.331px]" data-name="Text">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border h-[19.993px] relative w-[38.331px]">
        <p className="absolute font-['Inter:Regular',sans-serif] font-normal leading-[20px] left-0 not-italic text-[#888888] text-[14px] text-nowrap top-[0.36px] tracking-[-0.1504px] whitespace-pre">July 2</p>
      </div>
    </div>
  );
}

function Container41() {
  return (
    <div className="h-[19.993px] relative shrink-0 w-[58.324px]" data-name="Container">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border content-stretch flex gap-[5.994px] h-[19.993px] items-center relative w-[58.324px]">
        <Icon31 />
        <Text30 />
      </div>
    </div>
  );
}

function Icon32() {
  return (
    <div className="relative shrink-0 size-[13.999px]" data-name="Icon">
      <svg className="block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 14 14">
        <g clipPath="url(#clip0_16_2980)" id="Icon">
          <path d={svgPaths.p29305080} id="Vector" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.16655" />
        </g>
        <defs>
          <clipPath id="clip0_16_2980">
            <rect fill="white" height="13.9986" width="13.9986" />
          </clipPath>
        </defs>
      </svg>
    </div>
  );
}

function Text31() {
  return (
    <div className="h-[19.993px] relative shrink-0 w-[8.509px]" data-name="Text">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border h-[19.993px] relative w-[8.509px]">
        <p className="absolute font-['Inter:Regular',sans-serif] font-normal leading-[20px] left-0 not-italic text-[#888888] text-[14px] text-nowrap top-[0.36px] tracking-[-0.1504px] whitespace-pre">5</p>
      </div>
    </div>
  );
}

function Container42() {
  return (
    <div className="h-[19.993px] relative shrink-0 w-[28.501px]" data-name="Container">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border content-stretch flex gap-[5.994px] h-[19.993px] items-center relative w-[28.501px]">
        <Icon32 />
        <Text31 />
      </div>
    </div>
  );
}

function Icon33() {
  return (
    <div className="relative shrink-0 size-[13.999px]" data-name="Icon">
      <svg className="block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 14 14">
        <g clipPath="url(#clip0_16_2955)" id="Icon">
          <path d={svgPaths.p25e4e300} id="Vector" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.16655" />
        </g>
        <defs>
          <clipPath id="clip0_16_2955">
            <rect fill="white" height="13.9986" width="13.9986" />
          </clipPath>
        </defs>
      </svg>
    </div>
  );
}

function Text32() {
  return (
    <div className="h-[19.993px] relative shrink-0 w-[14.972px]" data-name="Text">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border h-[19.993px] relative w-[14.972px]">
        <p className="absolute font-['Inter:Regular',sans-serif] font-normal leading-[20px] left-0 not-italic text-[#888888] text-[14px] text-nowrap top-[0.36px] tracking-[-0.1504px] whitespace-pre">13</p>
      </div>
    </div>
  );
}

function Container43() {
  return (
    <div className="h-[19.993px] relative shrink-0 w-[34.965px]" data-name="Container">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border content-stretch flex gap-[5.994px] h-[19.993px] items-center relative w-[34.965px]">
        <Icon33 />
        <Text32 />
      </div>
    </div>
  );
}

function Container44() {
  return (
    <div className="absolute content-stretch flex gap-[15.994px] h-[19.993px] items-center left-[16.9px] top-[188.85px] w-[605.604px]" data-name="Container">
      <Container41 />
      <Container42 />
      <Container43 />
    </div>
  );
}

function KanbanCard4() {
  return (
    <div className="bg-[#1a1a1a] h-[225.746px] relative rounded-[10px] shrink-0 w-full" data-name="KanbanCard">
      <div aria-hidden="true" className="absolute border-[0.909px] border-neutral-800 border-solid inset-0 pointer-events-none rounded-[10px]" />
      <Container39 />
      <Heading7 />
      <Paragraph4 />
      <Container40 />
      <Container44 />
    </div>
  );
}

function Icon34() {
  return (
    <div className="relative shrink-0 size-[15.994px]" data-name="Icon">
      <svg className="block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 16 16">
        <g clipPath="url(#clip0_16_2997)" id="Icon">
          <path d={svgPaths.p25e9a960} id="Vector" stroke="var(--stroke-0, #54A0FF)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
          <path d={svgPaths.p1f683700} id="Vector_2" stroke="var(--stroke-0, #54A0FF)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
          <path d={svgPaths.p1b0f4080} id="Vector_3" stroke="var(--stroke-0, #54A0FF)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
          <path d={svgPaths.p247c7300} id="Vector_4" stroke="var(--stroke-0, #54A0FF)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
          <path d={svgPaths.pe8d1500} id="Vector_5" stroke="var(--stroke-0, #54A0FF)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
          <path d={svgPaths.pf60dd00} id="Vector_6" stroke="var(--stroke-0, #54A0FF)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
          <path d={svgPaths.pccac580} id="Vector_7" stroke="var(--stroke-0, #54A0FF)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
          <path d={svgPaths.p291c1f40} id="Vector_8" stroke="var(--stroke-0, #54A0FF)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
        </g>
        <defs>
          <clipPath id="clip0_16_2997">
            <rect fill="white" height="15.9943" width="15.9943" />
          </clipPath>
        </defs>
      </svg>
    </div>
  );
}

function Text33() {
  return (
    <div className="h-[19.993px] relative shrink-0 w-[51.747px]" data-name="Text">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border h-[19.993px] relative w-[51.747px]">
        <p className="absolute font-['Inter:Regular',sans-serif] font-normal leading-[20px] left-0 not-italic text-[#54a0ff] text-[14px] text-nowrap top-[0.36px] tracking-[-0.1504px] whitespace-pre">Medium</p>
      </div>
    </div>
  );
}

function StatusTag15() {
  return (
    <div className="absolute bg-[#1f2d3a] box-border content-stretch flex gap-[5.994px] h-[31.982px] items-center left-0 pl-[11.996px] pr-0 py-0 rounded-[1.5252e+07px] top-0 w-[97.727px]" data-name="StatusTag">
      <Icon34 />
      <Text33 />
    </div>
  );
}

function Text34() {
  return (
    <div className="h-[19.993px] relative shrink-0 w-[55.739px]" data-name="Text">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border h-[19.993px] relative w-[55.739px]">
        <p className="absolute font-['Inter:Regular',sans-serif] font-normal leading-[20px] left-0 not-italic text-[#888888] text-[14px] text-nowrap top-[0.36px] tracking-[-0.1504px] whitespace-pre">Backend</p>
      </div>
    </div>
  );
}

function StatusTag16() {
  return (
    <div className="absolute bg-neutral-800 box-border content-stretch flex h-[31.982px] items-center left-[105.72px] pl-[11.996px] pr-0 py-0 rounded-[1.5252e+07px] top-0 w-[79.73px]" data-name="StatusTag">
      <Text34 />
    </div>
  );
}

function Text35() {
  return (
    <div className="h-[19.993px] relative shrink-0 w-[30.98px]" data-name="Text">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border h-[19.993px] relative w-[30.98px]">
        <p className="absolute font-['Inter:Regular',sans-serif] font-normal leading-[20px] left-0 not-italic text-[#888888] text-[14px] text-nowrap top-[0.36px] tracking-[-0.1504px] whitespace-pre">Tech</p>
      </div>
    </div>
  );
}

function StatusTag17() {
  return (
    <div className="absolute bg-neutral-800 box-border content-stretch flex h-[31.982px] items-center left-[193.45px] pl-[11.996px] pr-0 py-0 rounded-[1.5252e+07px] top-0 w-[54.972px]" data-name="StatusTag">
      <Text35 />
    </div>
  );
}

function Container45() {
  return (
    <div className="absolute h-[31.982px] left-0 top-0 w-[248.423px]" data-name="Container">
      <StatusTag15 />
      <StatusTag16 />
      <StatusTag17 />
    </div>
  );
}

function Icon35() {
  return (
    <div className="relative shrink-0 size-[15.994px]" data-name="Icon">
      <svg className="block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 16 16">
        <g id="Icon">
          <path d={svgPaths.p65f2d00} id="Vector" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
          <path d={svgPaths.pb148ac0} id="Vector_2" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
          <path d={svgPaths.p19650e00} id="Vector_3" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
        </g>
      </svg>
    </div>
  );
}

function Button16() {
  return (
    <div className="absolute content-stretch flex items-center justify-center left-[589.6px] rounded-[8px] size-[20px] top-[-4px]" data-name="Button">
      <Icon35 />
    </div>
  );
}

function Container46() {
  return (
    <div className="absolute h-[31.982px] left-[16.9px] top-[16.9px] w-[605.604px]" data-name="Container">
      <Container45 />
      <Button16 />
    </div>
  );
}

function Heading8() {
  return (
    <div className="absolute h-[23.999px] left-[16.9px] top-[60.88px] w-[605.604px]" data-name="Heading 4">
      <p className="absolute font-['Inter:Regular',sans-serif] font-normal leading-[24px] left-0 not-italic text-[16px] text-nowrap text-white top-[-0.73px] tracking-[-0.3125px] whitespace-pre">Document API endpoints for task CRUD operations</p>
    </div>
  );
}

function Paragraph5() {
  return (
    <div className="absolute h-[39.986px] left-[16.9px] overflow-clip top-[92.88px] w-[605.604px]" data-name="Paragraph">
      <p className="absolute font-['Inter:Regular',sans-serif] font-normal leading-[20px] left-0 not-italic text-[#888888] text-[14px] top-[0.36px] tracking-[-0.1504px] w-[574px]">Create clear and structured documentation for all task-related Create, Read, Update, and Delete API.</p>
    </div>
  );
}

function PrimitiveImg10() {
  return (
    <div className="relative shrink-0 size-[20.362px]" data-name="Primitive.img">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border size-[20.362px]" />
    </div>
  );
}

function PrimitiveSpan10() {
  return (
    <div className="absolute left-0 rounded-[1.5252e+07px] size-[23.999px] top-0" data-name="Primitive.span">
      <div className="box-border content-stretch flex items-start overflow-clip p-[1.818px] relative rounded-[inherit] size-[23.999px]">
        <PrimitiveImg10 />
      </div>
      <div aria-hidden="true" className="absolute border-[#1a1a1a] border-[1.818px] border-solid inset-0 pointer-events-none rounded-[1.5252e+07px]" />
    </div>
  );
}

function PrimitiveImg11() {
  return (
    <div className="relative shrink-0 size-[20.362px]" data-name="Primitive.img">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border size-[20.362px]" />
    </div>
  );
}

function PrimitiveSpan11() {
  return (
    <div className="absolute left-[16px] rounded-[1.5252e+07px] size-[23.999px] top-0" data-name="Primitive.span">
      <div className="box-border content-stretch flex items-start overflow-clip p-[1.818px] relative rounded-[inherit] size-[23.999px]">
        <PrimitiveImg11 />
      </div>
      <div aria-hidden="true" className="absolute border-[#1a1a1a] border-[1.818px] border-solid inset-0 pointer-events-none rounded-[1.5252e+07px]" />
    </div>
  );
}

function Container47() {
  return (
    <div className="h-[23.999px] relative shrink-0 w-[40px]" data-name="Container">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border h-[23.999px] relative w-[40px]">
        <PrimitiveSpan10 />
        <PrimitiveSpan11 />
      </div>
    </div>
  );
}

function Icon36() {
  return (
    <div className="relative shrink-0 size-[15.994px]" data-name="Icon">
      <svg className="block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 16 16">
        <g id="Icon">
          <path d="M3.33215 7.99716H12.6622" id="Vector" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
          <path d="M7.99716 3.33215V12.6622" id="Vector_2" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
        </g>
      </svg>
    </div>
  );
}

function Button17() {
  return (
    <div className="relative rounded-[8px] shrink-0 size-[23.999px]" data-name="Button">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border content-stretch flex items-center justify-center pl-0 pr-[0.007px] py-0 relative size-[23.999px]">
        <Icon36 />
      </div>
    </div>
  );
}

function Container48() {
  return (
    <div className="absolute content-stretch flex gap-[3.999px] h-[23.999px] items-center left-[16.9px] top-[148.86px] w-[605.604px]" data-name="Container">
      <Container47 />
      <Button17 />
    </div>
  );
}

function Icon37() {
  return (
    <div className="relative shrink-0 size-[13.999px]" data-name="Icon">
      <svg className="block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 14 14">
        <g clipPath="url(#clip0_16_2991)" id="Icon">
          <path d="M4.66619 1.16655V3.49964" id="Vector" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.16655" />
          <path d="M9.33239 1.16655V3.49964" id="Vector_2" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.16655" />
          <path d={svgPaths.p3c05b400} id="Vector_3" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.16655" />
          <path d="M1.74982 5.83274H12.2488" id="Vector_4" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.16655" />
        </g>
        <defs>
          <clipPath id="clip0_16_2991">
            <rect fill="white" height="13.9986" width="13.9986" />
          </clipPath>
        </defs>
      </svg>
    </div>
  );
}

function Text36() {
  return (
    <div className="h-[19.993px] relative shrink-0 w-[38.537px]" data-name="Text">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border h-[19.993px] relative w-[38.537px]">
        <p className="absolute font-['Inter:Regular',sans-serif] font-normal leading-[20px] left-0 not-italic text-[#888888] text-[14px] text-nowrap top-[0.36px] tracking-[-0.1504px] whitespace-pre">July 5</p>
      </div>
    </div>
  );
}

function Container49() {
  return (
    <div className="h-[19.993px] relative shrink-0 w-[58.53px]" data-name="Container">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border content-stretch flex gap-[5.994px] h-[19.993px] items-center relative w-[58.53px]">
        <Icon37 />
        <Text36 />
      </div>
    </div>
  );
}

function Icon38() {
  return (
    <div className="relative shrink-0 size-[13.999px]" data-name="Icon">
      <svg className="block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 14 14">
        <g clipPath="url(#clip0_16_2980)" id="Icon">
          <path d={svgPaths.p29305080} id="Vector" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.16655" />
        </g>
        <defs>
          <clipPath id="clip0_16_2980">
            <rect fill="white" height="13.9986" width="13.9986" />
          </clipPath>
        </defs>
      </svg>
    </div>
  );
}

function Text37() {
  return (
    <div className="h-[19.993px] relative shrink-0 w-[17.493px]" data-name="Text">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border h-[19.993px] relative w-[17.493px]">
        <p className="absolute font-['Inter:Regular',sans-serif] font-normal leading-[20px] left-0 not-italic text-[#888888] text-[14px] text-nowrap top-[0.36px] tracking-[-0.1504px] whitespace-pre">34</p>
      </div>
    </div>
  );
}

function Container50() {
  return (
    <div className="h-[19.993px] relative shrink-0 w-[37.486px]" data-name="Container">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border content-stretch flex gap-[5.994px] h-[19.993px] items-center relative w-[37.486px]">
        <Icon38 />
        <Text37 />
      </div>
    </div>
  );
}

function Icon39() {
  return (
    <div className="relative shrink-0 size-[13.999px]" data-name="Icon">
      <svg className="block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 14 14">
        <g clipPath="url(#clip0_16_2983)" id="Icon">
          <path d={svgPaths.p210e4b84} id="Vector" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.16655" />
        </g>
        <defs>
          <clipPath id="clip0_16_2983">
            <rect fill="white" height="13.9986" width="13.9986" />
          </clipPath>
        </defs>
      </svg>
    </div>
  );
}

function Text38() {
  return (
    <div className="h-[19.993px] relative shrink-0 w-[6.349px]" data-name="Text">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border h-[19.993px] relative w-[6.349px]">
        <p className="absolute font-['Inter:Regular',sans-serif] font-normal leading-[20px] left-0 not-italic text-[#888888] text-[14px] text-nowrap top-[0.36px] tracking-[-0.1504px] whitespace-pre">1</p>
      </div>
    </div>
  );
}

function Container51() {
  return (
    <div className="h-[19.993px] relative shrink-0 w-[26.342px]" data-name="Container">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border content-stretch flex gap-[5.994px] h-[19.993px] items-center relative w-[26.342px]">
        <Icon39 />
        <Text38 />
      </div>
    </div>
  );
}

function Container52() {
  return (
    <div className="absolute content-stretch flex gap-[15.994px] h-[19.993px] items-center left-[16.9px] top-[188.85px] w-[605.604px]" data-name="Container">
      <Container49 />
      <Container50 />
      <Container51 />
    </div>
  );
}

function KanbanCard5() {
  return (
    <div className="bg-[#1a1a1a] h-[225.746px] relative rounded-[10px] shrink-0 w-full" data-name="KanbanCard">
      <div aria-hidden="true" className="absolute border-[0.909px] border-neutral-800 border-solid inset-0 pointer-events-none rounded-[10px]" />
      <Container46 />
      <Heading8 />
      <Paragraph5 />
      <Container48 />
      <Container52 />
    </div>
  );
}

function Container53() {
  return (
    <div className="h-[463.487px] relative shrink-0 w-[639.41px]" data-name="Container">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border content-stretch flex flex-col gap-[11.996px] h-[463.487px] items-start relative w-[639.41px]">
        <KanbanCard4 />
        <KanbanCard5 />
      </div>
    </div>
  );
}

function KanbanColumn2() {
  return (
    <div className="absolute content-stretch flex flex-col gap-[15.994px] h-[735.043px] items-start left-[1310.82px] top-0 w-[639.41px]" data-name="KanbanColumn">
      <Container37 />
      <Button13 status="in-progress" />
      <Container53 />
    </div>
  );
}

function Heading9() {
  return (
    <div className="h-[23.999px] relative shrink-0 w-[38.31px]" data-name="Heading 3">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border h-[23.999px] relative w-[38.31px]">
        <p className="absolute font-['Inter:Regular',sans-serif] font-normal leading-[24px] left-0 not-italic text-[16px] text-nowrap text-white top-[-0.73px] tracking-[-0.3125px] whitespace-pre">Done</p>
      </div>
    </div>
  );
}

function Text39() {
  return (
    <div className="h-[19.993px] relative shrink-0 w-[48.189px]" data-name="Text">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border h-[19.993px] relative w-[48.189px]">
        <p className="absolute font-['Inter:Regular',sans-serif] font-normal leading-[20px] left-0 not-italic text-[#888888] text-[14px] top-[0.36px] tracking-[-0.1504px] w-[49px]">1 Cards</p>
      </div>
    </div>
  );
}

function Container54() {
  return (
    <div className="h-[23.999px] relative shrink-0 w-[94.496px]" data-name="Container">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border content-stretch flex gap-[7.997px] h-[23.999px] items-center relative w-[94.496px]">
        <Heading9 />
        <Text39 />
      </div>
    </div>
  );
}

function Icon40() {
  return (
    <div className="relative shrink-0 size-[15.994px]" data-name="Icon">
      <svg className="block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 16 16">
        <g id="Icon">
          <path d={svgPaths.p65f2d00} id="Vector" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
          <path d={svgPaths.pb148ac0} id="Vector_2" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
          <path d={svgPaths.p19650e00} id="Vector_3" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
        </g>
      </svg>
    </div>
  );
}

function Button18() {
  return (
    <div className="relative rounded-[8px] shrink-0 size-[23.999px]" data-name="Button">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border content-stretch flex items-center justify-center pl-0 pr-[0.007px] py-0 relative size-[23.999px]">
        <Icon40 />
      </div>
    </div>
  );
}

function Container55() {
  return (
    <div className="h-[23.999px] relative shrink-0 w-[639.418px]" data-name="Container">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border content-stretch flex h-[23.999px] items-center justify-between relative w-[639.418px]">
        <Container54 />
        <Button18 />
      </div>
    </div>
  );
}

function Icon41() {
  return (
    <div className="relative shrink-0 size-[15.994px]" data-name="Icon">
      <svg className="block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 16 16">
        <g id="Icon">
          <path d="M3.33215 7.99716H12.6622" id="Vector" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
          <path d="M7.99716 3.33215V12.6622" id="Vector_2" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
        </g>
      </svg>
    </div>
  );
}

function Button19({ status }: { status?: string }) {
  return (
    <button 
      className="h-[41.804px] relative rounded-[10px] shrink-0 w-[639.418px] hover:bg-[#1a1a1a] transition-colors cursor-pointer" 
      data-name="Button"
      data-add-task="true"
      data-status={status}
    >
      <div aria-hidden="true" className="absolute border-[0.909px] border-neutral-800 border-solid inset-0 pointer-events-none rounded-[10px]" />
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border content-stretch flex h-[41.804px] items-center justify-center p-[0.909px] relative w-[639.418px]">
        <Icon41 />
      </div>
    </button>
  );
}

function Icon42() {
  return (
    <div className="relative shrink-0 size-[15.994px]" data-name="Icon">
      <svg className="block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 16 16">
        <g clipPath="url(#clip0_16_2962)" id="Icon">
          <path d={svgPaths.p3c8e7f00} id="Vector" stroke="var(--stroke-0, #FF9F43)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
          <path d="M7.99716 5.99787V8.66359" id="Vector_2" stroke="var(--stroke-0, #FF9F43)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
          <path d="M7.99716 11.3293H8.00382" id="Vector_3" stroke="var(--stroke-0, #FF9F43)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
        </g>
        <defs>
          <clipPath id="clip0_16_2962">
            <rect fill="white" height="15.9943" width="15.9943" />
          </clipPath>
        </defs>
      </svg>
    </div>
  );
}

function Text40() {
  return (
    <div className="h-[19.993px] relative shrink-0 w-[30.021px]" data-name="Text">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border h-[19.993px] relative w-[30.021px]">
        <p className="absolute font-['Inter:Regular',sans-serif] font-normal leading-[20px] left-0 not-italic text-[#ff9f43] text-[14px] text-nowrap top-[0.36px] tracking-[-0.1504px] whitespace-pre">High</p>
      </div>
    </div>
  );
}

function StatusTag18() {
  return (
    <div className="absolute bg-[#3a2f1f] box-border content-stretch flex gap-[5.994px] h-[31.982px] items-center left-0 pl-[11.996px] pr-0 py-0 rounded-[1.5252e+07px] top-0 w-[76.001px]" data-name="StatusTag">
      <Icon42 />
      <Text40 />
    </div>
  );
}

function Text41() {
  return (
    <div className="h-[19.993px] relative shrink-0 w-[59.986px]" data-name="Text">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border h-[19.993px] relative w-[59.986px]">
        <p className="absolute font-['Inter:Regular',sans-serif] font-normal leading-[20px] left-0 not-italic text-[#888888] text-[14px] text-nowrap top-[0.36px] tracking-[-0.1504px] whitespace-pre">Research</p>
      </div>
    </div>
  );
}

function StatusTag19() {
  return (
    <div className="absolute bg-neutral-800 box-border content-stretch flex h-[31.982px] items-center left-[84px] pl-[11.996px] pr-0 py-0 rounded-[1.5252e+07px] top-0 w-[83.977px]" data-name="StatusTag">
      <Text41 />
    </div>
  );
}

function Text42() {
  return (
    <div className="h-[19.993px] relative shrink-0 w-[33.658px]" data-name="Text">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border h-[19.993px] relative w-[33.658px]">
        <p className="absolute font-['Inter:Regular',sans-serif] font-normal leading-[20px] left-0 not-italic text-[#888888] text-[14px] text-nowrap top-[0.36px] tracking-[-0.1504px] whitespace-pre">Audit</p>
      </div>
    </div>
  );
}

function StatusTag20() {
  return (
    <div className="absolute bg-neutral-800 box-border content-stretch flex h-[31.982px] items-center left-[175.97px] pl-[11.996px] pr-0 py-0 rounded-[1.5252e+07px] top-0 w-[57.649px]" data-name="StatusTag">
      <Text42 />
    </div>
  );
}

function Container56() {
  return (
    <div className="absolute h-[31.982px] left-0 top-0 w-[233.622px]" data-name="Container">
      <StatusTag18 />
      <StatusTag19 />
      <StatusTag20 />
    </div>
  );
}

function Icon43() {
  return (
    <div className="relative shrink-0 size-[15.994px]" data-name="Icon">
      <svg className="block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 16 16">
        <g id="Icon">
          <path d={svgPaths.p65f2d00} id="Vector" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
          <path d={svgPaths.pb148ac0} id="Vector_2" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
          <path d={svgPaths.p19650e00} id="Vector_3" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
        </g>
      </svg>
    </div>
  );
}

function Button20() {
  return (
    <div className="absolute content-stretch flex items-center justify-center left-[589.61px] rounded-[8px] size-[20px] top-[-4px]" data-name="Button">
      <Icon43 />
    </div>
  );
}

function Container57() {
  return (
    <div className="absolute h-[31.982px] left-[16.9px] top-[16.9px] w-[605.611px]" data-name="Container">
      <Container56 />
      <Button20 />
    </div>
  );
}

function Heading10() {
  return (
    <div className="absolute h-[23.999px] left-[16.9px] top-[60.88px] w-[605.611px]" data-name="Heading 4">
      <p className="absolute font-['Inter:Regular',sans-serif] font-normal leading-[24px] left-0 not-italic text-[16px] text-nowrap text-white top-[-0.73px] tracking-[-0.3125px] whitespace-pre">Create initial column layout (To Do, In Progress, Done)</p>
    </div>
  );
}

function Paragraph6() {
  return (
    <div className="absolute h-[19.993px] left-[16.9px] overflow-clip top-[92.88px] w-[605.611px]" data-name="Paragraph">
      <p className="absolute font-['Inter:Regular',sans-serif] font-normal leading-[20px] left-0 not-italic text-[#888888] text-[14px] text-nowrap top-[0.36px] tracking-[-0.1504px] whitespace-pre">Set up the foundational kanban board structure with properly spaced columns and styling.</p>
    </div>
  );
}

function PrimitiveImg12() {
  return (
    <div className="relative shrink-0 size-[20.362px]" data-name="Primitive.img">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border size-[20.362px]" />
    </div>
  );
}

function PrimitiveSpan12() {
  return (
    <div className="absolute left-0 rounded-[1.5252e+07px] size-[23.999px] top-0" data-name="Primitive.span">
      <div className="box-border content-stretch flex items-start overflow-clip p-[1.818px] relative rounded-[inherit] size-[23.999px]">
        <PrimitiveImg12 />
      </div>
      <div aria-hidden="true" className="absolute border-[#1a1a1a] border-[1.818px] border-solid inset-0 pointer-events-none rounded-[1.5252e+07px]" />
    </div>
  );
}

function PrimitiveImg13() {
  return (
    <div className="relative shrink-0 size-[20.362px]" data-name="Primitive.img">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border size-[20.362px]" />
    </div>
  );
}

function PrimitiveSpan13() {
  return (
    <div className="absolute left-[16px] rounded-[1.5252e+07px] size-[23.999px] top-0" data-name="Primitive.span">
      <div className="box-border content-stretch flex items-start overflow-clip p-[1.818px] relative rounded-[inherit] size-[23.999px]">
        <PrimitiveImg13 />
      </div>
      <div aria-hidden="true" className="absolute border-[#1a1a1a] border-[1.818px] border-solid inset-0 pointer-events-none rounded-[1.5252e+07px]" />
    </div>
  );
}

function PrimitiveImg14() {
  return (
    <div className="relative shrink-0 size-[20.362px]" data-name="Primitive.img">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border size-[20.362px]" />
    </div>
  );
}

function PrimitiveSpan14() {
  return (
    <div className="absolute left-[32px] rounded-[1.5252e+07px] size-[23.999px] top-0" data-name="Primitive.span">
      <div className="box-border content-stretch flex items-start overflow-clip p-[1.818px] relative rounded-[inherit] size-[23.999px]">
        <PrimitiveImg14 />
      </div>
      <div aria-hidden="true" className="absolute border-[#1a1a1a] border-[1.818px] border-solid inset-0 pointer-events-none rounded-[1.5252e+07px]" />
    </div>
  );
}

function Container58() {
  return (
    <div className="h-[23.999px] relative shrink-0 w-[56.001px]" data-name="Container">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border h-[23.999px] relative w-[56.001px]">
        <PrimitiveSpan12 />
        <PrimitiveSpan13 />
        <PrimitiveSpan14 />
      </div>
    </div>
  );
}

function Icon44() {
  return (
    <div className="relative shrink-0 size-[15.994px]" data-name="Icon">
      <svg className="block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 16 16">
        <g id="Icon">
          <path d="M3.33215 7.99716H12.6622" id="Vector" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
          <path d="M7.99716 3.33215V12.6622" id="Vector_2" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
        </g>
      </svg>
    </div>
  );
}

function Button21() {
  return (
    <div className="relative rounded-[8px] shrink-0 size-[23.999px]" data-name="Button">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border content-stretch flex items-center justify-center pl-0 pr-[0.007px] py-0 relative size-[23.999px]">
        <Icon44 />
      </div>
    </div>
  );
}

function Container59() {
  return (
    <div className="absolute content-stretch flex gap-[3.999px] h-[23.999px] items-center left-[16.9px] top-[128.86px] w-[605.611px]" data-name="Container">
      <Container58 />
      <Button21 />
    </div>
  );
}

function Icon45() {
  return (
    <div className="relative shrink-0 size-[13.999px]" data-name="Icon">
      <svg className="block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 14 14">
        <g clipPath="url(#clip0_16_2991)" id="Icon">
          <path d="M4.66619 1.16655V3.49964" id="Vector" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.16655" />
          <path d="M9.33239 1.16655V3.49964" id="Vector_2" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.16655" />
          <path d={svgPaths.p3c05b400} id="Vector_3" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.16655" />
          <path d="M1.74982 5.83274H12.2488" id="Vector_4" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.16655" />
        </g>
        <defs>
          <clipPath id="clip0_16_2991">
            <rect fill="white" height="13.9986" width="13.9986" />
          </clipPath>
        </defs>
      </svg>
    </div>
  );
}

function Text43() {
  return (
    <div className="h-[19.993px] relative shrink-0 w-[38.331px]" data-name="Text">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border h-[19.993px] relative w-[38.331px]">
        <p className="absolute font-['Inter:Regular',sans-serif] font-normal leading-[20px] left-0 not-italic text-[#888888] text-[14px] text-nowrap top-[0.36px] tracking-[-0.1504px] whitespace-pre">July 2</p>
      </div>
    </div>
  );
}

function Container60() {
  return (
    <div className="h-[19.993px] relative shrink-0 w-[58.324px]" data-name="Container">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border content-stretch flex gap-[5.994px] h-[19.993px] items-center relative w-[58.324px]">
        <Icon45 />
        <Text43 />
      </div>
    </div>
  );
}

function Icon46() {
  return (
    <div className="relative shrink-0 size-[13.999px]" data-name="Icon">
      <svg className="block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 14 14">
        <g clipPath="url(#clip0_16_2980)" id="Icon">
          <path d={svgPaths.p29305080} id="Vector" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.16655" />
        </g>
        <defs>
          <clipPath id="clip0_16_2980">
            <rect fill="white" height="13.9986" width="13.9986" />
          </clipPath>
        </defs>
      </svg>
    </div>
  );
}

function Text44() {
  return (
    <div className="h-[19.993px] relative shrink-0 w-[17.365px]" data-name="Text">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border h-[19.993px] relative w-[17.365px]">
        <p className="absolute font-['Inter:Regular',sans-serif] font-normal leading-[20px] left-0 not-italic text-[#888888] text-[14px] text-nowrap top-[0.36px] tracking-[-0.1504px] whitespace-pre">45</p>
      </div>
    </div>
  );
}

function Container61() {
  return (
    <div className="h-[19.993px] relative shrink-0 w-[37.358px]" data-name="Container">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border content-stretch flex gap-[5.994px] h-[19.993px] items-center relative w-[37.358px]">
        <Icon46 />
        <Text44 />
      </div>
    </div>
  );
}

function Icon47() {
  return (
    <div className="relative shrink-0 size-[13.999px]" data-name="Icon">
      <svg className="block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 14 14">
        <g clipPath="url(#clip0_16_2955)" id="Icon">
          <path d={svgPaths.p25e4e300} id="Vector" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.16655" />
        </g>
        <defs>
          <clipPath id="clip0_16_2955">
            <rect fill="white" height="13.9986" width="13.9986" />
          </clipPath>
        </defs>
      </svg>
    </div>
  );
}

function Text45() {
  return (
    <div className="h-[19.993px] relative shrink-0 w-[8.793px]" data-name="Text">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border h-[19.993px] relative w-[8.793px]">
        <p className="absolute font-['Inter:Regular',sans-serif] font-normal leading-[20px] left-0 not-italic text-[#888888] text-[14px] text-nowrap top-[0.36px] tracking-[-0.1504px] whitespace-pre">8</p>
      </div>
    </div>
  );
}

function Container62() {
  return (
    <div className="h-[19.993px] relative shrink-0 w-[28.785px]" data-name="Container">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border content-stretch flex gap-[5.994px] h-[19.993px] items-center relative w-[28.785px]">
        <Icon47 />
        <Text45 />
      </div>
    </div>
  );
}

function Container63() {
  return (
    <div className="absolute content-stretch flex gap-[15.994px] h-[19.993px] items-center left-[16.9px] top-[168.86px] w-[605.611px]" data-name="Container">
      <Container60 />
      <Container61 />
      <Container62 />
    </div>
  );
}

function KanbanCard6() {
  return (
    <div className="bg-[#1a1a1a] h-[205.753px] relative rounded-[10px] shrink-0 w-[639.418px]" data-name="KanbanCard">
      <div aria-hidden="true" className="absolute border-[0.909px] border-neutral-800 border-solid inset-0 pointer-events-none rounded-[10px]" />
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border h-[205.753px] relative w-[639.418px]">
        <Container57 />
        <Heading10 />
        <Paragraph6 />
        <Container59 />
        <Container63 />
      </div>
    </div>
  );
}

function KanbanColumn3() {
  return (
    <div className="absolute content-stretch flex flex-col gap-[15.994px] h-[735.043px] items-start left-[1966.22px] top-0 w-[639.418px]" data-name="KanbanColumn">
      <Container55 />
      <Button19 status="done" />
      <KanbanCard6 />
    </div>
  );
}

function Container64() {
  return (
    <div className="h-[735.043px] relative shrink-0 w-full" data-name="Container">
      <KanbanColumn />
      <KanbanColumn1 />
      <KanbanColumn2 />
      <KanbanColumn3 />
    </div>
  );
}

function Container65() {
  return (
    <div className="h-[783.04px] relative shrink-0 w-full" data-name="Container">
      <div className="overflow-clip rounded-[inherit] size-full">
        <div className="box-border content-stretch flex flex-col h-[783.04px] items-start pb-0 pl-[23.999px] pr-[23.998px] pt-[23.999px] relative w-full">
          <Container64 />
        </div>
      </div>
    </div>
  );
}

export default function Container66() {
  return (
    <div className="content-stretch flex flex-col items-start relative size-full" data-name="Container">
      <Container65 />
    </div>
  );
}