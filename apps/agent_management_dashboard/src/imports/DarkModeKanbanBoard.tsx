import svgPaths from "./svg-ustevohwso";
import styles from "./DarkModeKanbanBoard.module.scss";

function Paragraph() {
  return (
    <div className={`${styles.h19993px} ${styles.relative} ${styles.shrink0} ${styles.wFull}`} data-name="Paragraph">
      <p className={`${styles.absolute} ${styles.fontNormal} ${styles.leading20px} ${styles.left0} ${styles.notItalic} ${styles.text888888} ${styles.text14px} ${styles.textNowrap} ${styles.top036px} ${styles.trackingNeg1504px} ${styles.whitespacePre}`}>Projects</p>
    </div>
  );
}

function Heading() {
  return (
    <div className={`${styles.h31996px} ${styles.relative} ${styles.shrink0} ${styles.wFull}`} data-name="Heading 1">
      <p className={`${styles.absolute} ${styles.fontNormal} ${styles.leading32px} ${styles.left0} ${styles.notItalic} ${styles.text24px} ${styles.textNowrap} ${styles.textWhite} ${styles.topNeg018px} ${styles.tracking00703px} ${styles.whitespacePre}`}>Project name</p>
    </div>
  );
}

function Container() {
  return (
    <div className={`${styles.contentStretch} ${styles.flex} ${styles.flexCol} ${styles.gap3999px} ${styles.h55987px} ${styles.itemsStart} ${styles.relative} ${styles.shrink0} ${styles.wFull}`} data-name="Container">
      <Paragraph />
      <Heading />
    </div>
  );
}

function Button() {
  return (
    <div className={`${styles.absolute} ${styles.h35994px} ${styles.left0} ${styles.top0} ${styles.w67678px}`} data-name="Button">
      <p className={`${styles.absolute} ${styles.fontNormal} ${styles.leading24px} ${styles.left0} ${styles.notItalic} ${styles.text888888} ${styles.text16px} ${styles.textNowrap} ${styles.topNeg073px} ${styles.trackingNeg3125px} ${styles.whitespacePre}`}>Overview</p>
    </div>
  );
}

function Button1() {
  return (
    <div className={`${styles.absolute} ${styles.h35994px} ${styles.left9168px} ${styles.top0} ${styles.w80881px}`} data-name="Button">
      <p className={`${styles.absolute} ${styles.fontNormal} ${styles.leading24px} ${styles.left0} ${styles.notItalic} ${styles.text888888} ${styles.text16px} ${styles.textNowrap} ${styles.topNeg073px} ${styles.trackingNeg3125px} ${styles.whitespacePre}`}>Workspace</p>
    </div>
  );
}

function Container1() {
  return <div className={`${styles.absolute} ${styles.bgWhite} ${styles.h1996px} ${styles.left0} ${styles.top34px} ${styles.w40895px}`} data-name="Container" />;
}

function Button2() {
  return (
    <div className={`${styles.absolute} ${styles.h35994px} ${styles.left19656px} ${styles.top0} ${styles.w40895px}`} data-name="Button">
      <p className={`${styles.absolute} ${styles.fontNormal} ${styles.leading24px} ${styles.left0} ${styles.notItalic} ${styles.text16px} ${styles.textNowrap} ${styles.textWhite} ${styles.topNeg073px} ${styles.trackingNeg3125px} ${styles.whitespacePre}`}>Tasks</p>
      <Container1 />
    </div>
  );
}

function Button3() {
  return (
    <div className={`${styles.absolute} ${styles.h35994px} ${styles.left26145px} ${styles.top0} ${styles.w6098px}`} data-name="Button">
      <p className={`${styles.absolute} ${styles.fontNormal} ${styles.leading24px} ${styles.left0} ${styles.notItalic} ${styles.text888888} ${styles.text16px} ${styles.textNowrap} ${styles.topNeg073px} ${styles.trackingNeg3125px} ${styles.whitespacePre}`}>Timeline</p>
    </div>
  );
}

function Button4() {
  return (
    <div className={`${styles.absolute} ${styles.h35994px} ${styles.left34643px} ${styles.top0} ${styles.w11348px}`} data-name="Button">
      <p className={`${styles.absolute} ${styles.fontNormal} ${styles.leading24px} ${styles.left0} ${styles.notItalic} ${styles.text888888} ${styles.text16px} ${styles.textNowrap} ${styles.topNeg073px} ${styles.trackingNeg3125px} ${styles.whitespacePre}`}>Manage Project</p>
    </div>
  );
}

function Container2() {
  return (
    <div className={`${styles.h35994px} ${styles.relative} ${styles.shrink0} ${styles.w459908px}`} data-name="Container">
      <div className={`${styles.bgClipPadding} ${styles.border0} ${styles.borderTransparent} ${styles.borderSolid} ${styles.boxBorder} ${styles.h35994px} ${styles.relative} ${styles.w459908px}`}>
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
    <div className={`${styles.absolute} ${styles.left9619px} ${styles.size15994px} ${styles.top10px}`} data-name="Icon">
      <svg className={`${styles.block} ${styles.sizeFull}`} fill="none" preserveAspectRatio="none" viewBox="0 0 16 16">
        <g id="Icon">
          <path d={svgPaths.p10a02b40} id="Vector" stroke="var(--stroke-0, white)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
        </g>
      </svg>
    </div>
  );
}

function Button5() {
  return (
    <div className={`${styles.absolute} ${styles.bg1a1a1a} ${styles.h35994px} ${styles.left26799px} ${styles.rounded8px} ${styles.top0} ${styles.w125092px}`} data-name="Button">
      <div aria-hidden="true" className={`${styles.absolute} ${styles.border0909px} ${styles.borderNeutral800} ${styles.borderSolid} ${styles.inset0} ${styles.pointerEventsNone} ${styles.rounded8px}`} />
      <p className={`${styles.absolute} ${styles.fontMedium} ${styles.leading20px} ${styles.left129px} ${styles.notItalic} ${styles.text14px} ${styles.textNowrap} ${styles.textWhite} ${styles.top837px} ${styles.trackingNeg1504px} ${styles.whitespacePre}`}>Status: All</p>
      <Icon />
    </div>
  );
}

function Icon1() {
  return (
    <div className={`${styles.absolute} ${styles.left129px} ${styles.size15994px} ${styles.top10px}`} data-name="Icon">
      <svg className={`${styles.block} ${styles.sizeFull}`} fill="none" preserveAspectRatio="none" viewBox="0 0 16 16">
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
    <div className={`${styles.absolute} ${styles.bg1a1a1a} ${styles.h35994px} ${styles.left40508px} ${styles.rounded8px} ${styles.top0} ${styles.w8576px}`} data-name="Button">
      <div aria-hidden="true" className={`${styles.absolute} ${styles.border0909px} ${styles.borderNeutral800} ${styles.borderSolid} ${styles.inset0} ${styles.pointerEventsNone} ${styles.rounded8px}`} />
      <Icon1 />
      <p className={`${styles.absolute} ${styles.fontMedium} ${styles.leading20px} ${styles.left4489px} ${styles.notItalic} ${styles.text14px} ${styles.textNowrap} ${styles.textWhite} ${styles.top837px} ${styles.trackingNeg1504px} ${styles.whitespacePre}`}>Sort</p>
    </div>
  );
}

function Icon2() {
  return (
    <div className={`${styles.relative} ${styles.shrink0} ${styles.size15994px}`} data-name="Icon">
      <svg className={`${styles.block} ${styles.sizeFull}`} fill="none" preserveAspectRatio="none" viewBox="0 0 16 16">
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
    <div className={`${styles.absolute} ${styles.bg1a1a1a} ${styles.boxBorder} ${styles.contentStretch} ${styles.flex} ${styles.h35994px} ${styles.itemsCenter} ${styles.justifyCenter} ${styles.left50283px} ${styles.p0909px} ${styles.rounded8px} ${styles.top0} ${styles.w41804px}`} data-name="Button">
      <div aria-hidden="true" className={`${styles.absolute} ${styles.border0909px} ${styles.borderNeutral800} ${styles.borderSolid} ${styles.inset0} ${styles.pointerEventsNone} ${styles.rounded8px}`} />
      <Icon2 />
    </div>
  );
}

function Input() {
  return (
    <div className={`${styles.absolute} ${styles.bg1a1a1a} ${styles.h35994px} ${styles.left0} ${styles.rounded8px} ${styles.top0} ${styles.w255994px}`} data-name="Input">
      <div className={`${styles.boxBorder} ${styles.contentStretch} ${styles.flex} ${styles.h35994px} ${styles.itemsCenter} ${styles.overflowClip} ${styles.pl36px} ${styles.pr48px} ${styles.py4px} ${styles.relative} ${styles.roundedInherit} ${styles.w255994px}`}>
        <p className={`${styles.fontNormal} ${styles.leadingNormal} ${styles.notItalic} ${styles.relative} ${styles.shrink0} ${styles.text888888} ${styles.text14px} ${styles.textNowrap} ${styles.trackingNeg1504px} ${styles.whitespacePre}`}>Search</p>
      </div>
      <div aria-hidden="true" className={`${styles.absolute} ${styles.border0909px} ${styles.borderNeutral800} ${styles.borderSolid} ${styles.inset0} ${styles.pointerEventsNone} ${styles.rounded8px}`} />
    </div>
  );
}

function Icon3() {
  return (
    <div className={`${styles.absolute} ${styles.left12px} ${styles.size15994px} ${styles.top10px}`} data-name="Icon">
      <svg className={`${styles.block} ${styles.sizeFull}`} fill="none" preserveAspectRatio="none" viewBox="0 0 16 16">
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
    <div className={`${styles.absolute} ${styles.h16001px} ${styles.left22653px} ${styles.top10px} ${styles.w17465px}`} data-name="Text">
      <p className={`${styles.absolute} ${styles.fontNormal} ${styles.leading16px} ${styles.left0} ${styles.notItalic} ${styles.text888888} ${styles.text12px} ${styles.textNowrap} ${styles.top046px} ${styles.whitespacePre}`}>⌘F</p>
    </div>
  );
}

function Container3() {
  return (
    <div className={`${styles.absolute} ${styles.h35994px} ${styles.left0} ${styles.top0} ${styles.w255994px}`} data-name="Container">
      <Input />
      <Icon3 />
      <Text />
    </div>
  );
}

function Container4() {
  return (
    <div className={`${styles.h35994px} ${styles.relative} ${styles.shrink0} ${styles.w544638px}`} data-name="Container">
      <div className={`${styles.bgClipPadding} ${styles.border0} ${styles.borderTransparent} ${styles.borderSolid} ${styles.boxBorder} ${styles.h35994px} ${styles.relative} ${styles.w544638px}`}>
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
    <div className={`${styles.contentStretch} ${styles.flex} ${styles.h35994px} ${styles.itemsCenter} ${styles.justifyBetween} ${styles.relative} ${styles.shrink0} ${styles.wFull}`} data-name="Container">
      <Container2 />
      <Container4 />
    </div>
  );
}

function Container6() {
  return (
    <div className={`${styles.h140874px} ${styles.relative} ${styles.shrink0} ${styles.w265364px}`} data-name="Container">
      <div aria-hidden="true" className={`${styles.absolute} ${styles.border0_0_0909px} ${styles.borderNeutral800} ${styles.borderSolid} ${styles.inset0} ${styles.pointerEventsNone}`} />
      <div className={`${styles.bgClipPadding} ${styles.border0} ${styles.borderTransparent} ${styles.borderSolid} ${styles.boxBorder} ${styles.contentStretch} ${styles.flex} ${styles.flexCol} ${styles.gap15994px} ${styles.h140874px} ${styles.itemsStart} ${styles.pb0909px} ${styles.pl23999px} ${styles.pr23998px} ${styles.pt15994px} ${styles.relative} ${styles.w265364px}`}>
        <Container />
        <Container5 />
      </div>
    </div>
  );
}

export default function DarkModeKanbanBoard() {
  return (
    <div className={`${styles.bg0d0d0d} ${styles.contentStretch} ${styles.flex} ${styles.flexCol} ${styles.itemsStart} ${styles.relative} ${styles.sizeFull}`} data-name="Dark Mode Kanban Board">
      <Container6 />
    </div>
  );
}