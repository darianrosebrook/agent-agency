import svgPaths from "./svg-23263o4pzr";
import styles from "../components/projects/WorkspaceTab.module.scss";

interface HeadingProps {
  title: string;
}

function Heading({ title }: HeadingProps) {
  return (
    <div className={`${styles.relative} ${styles.shrink0}`} style={{ height: "23.991px" }} data-name="Heading 3">
      <div className={`${styles.bgClipPadding} ${styles.border0} ${styles.borderTransparent} ${styles.borderSolid} ${styles.boxBorder} ${styles.relative}`}
        style={{ height: "23.991px" }}
      >
        <p className={`${styles.fontNormal} ${styles.leading24} ${styles.notItalic} ${styles.text16} ${styles.textWhite} ${styles.trackingNeg3125}`}>{title}</p>
      </div>
    </div>
  );
}

function Icon() {
  return (
    <div className={`${styles.relative} ${styles.shrink0} ${styles.sizeIcon}`} data-name="Icon">
      <svg className={`${styles.block} ${styles.sizeFull}`} fill="none" preserveAspectRatio="none" viewBox="0 0 16 16">
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
    <button onClick={onClick} className={`${styles.relative} ${styles.rounded8} ${styles.shrink0} ${styles.sizeButton} ${styles.workspacePanelCloseButton}`} data-name="Button">
      <div className={`${styles.bgClipPadding} ${styles.border0} ${styles.borderTransparent} ${styles.borderSolid} ${styles.boxBorder} ${styles.contentStretch} ${styles.flex} ${styles.itemsCenter} ${styles.justifyCenter} ${styles.relative} ${styles.sizeButton}`}>
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
    <div className={`${styles.relative} ${styles.shrink0} ${styles.wFull}`} style={{ height: "64.886px" }} data-name="Container">
      <div aria-hidden="true" className={`${styles.absolute} ${styles.borderSolid} ${styles.inset0} ${styles.pointerEventsNone}`}
        style={{ borderWidth: "0px 0px 0.909px", borderColor: "#262626" }}
      />
      <div className={`${styles.flex} ${styles.flexRow} ${styles.itemsCenter} ${styles.sizeFull}`}>
        <div className={`${styles.boxBorder} ${styles.contentStretch} ${styles.flex} ${styles.itemsCenter} ${styles.justifyBetween} ${styles.relative} ${styles.wFull}`}
          style={{ height: "64.886px", paddingBottom: "0.909px", paddingLeft: "23.991px", paddingRight: "23.992px", paddingTop: 0 }}
        >
          <Heading title={title} />
          <Button onClick={onClose} />
        </div>
      </div>
    </div>
  );
}

function Icon1() {
  return (
    <div className={`${styles.relative} ${styles.shrink0}`} style={{ width: "31.989px", height: "31.989px" }} data-name="Icon">
      <svg className={`${styles.block} ${styles.sizeFull}`} fill="none" preserveAspectRatio="none" viewBox="0 0 32 32">
        <g id="Icon">
          <path d={svgPaths.p314f200} id="Vector" stroke="var(--stroke-0, #888888)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.99929" />
        </g>
      </svg>
    </div>
  );
}

function Container1() {
  return (
    <div className={`${styles.bgDarkSecondary} ${styles.boxBorder} ${styles.contentStretch} ${styles.flex} ${styles.itemsCenter} ${styles.justifyCenter} ${styles.relative} ${styles.shrink0}`}
      style={{ backgroundColor: "#1a1a1a", paddingLeft: "0.909px", paddingRight: "0.923px", paddingBlock: "0.909px", borderRadius: "16px", width: "63.992px", height: "63.992px" }}
      data-name="Container"
    >
      <div aria-hidden="true" className={`${styles.absolute} ${styles.borderSolid} ${styles.inset0} ${styles.pointerEventsNone}`}
        style={{ borderWidth: "0.909px", borderColor: "#262626", borderRadius: "16px" }}
      />
      <Icon1 />
    </div>
  );
}

function Heading1() {
  return (
    <div className={`${styles.relative} ${styles.shrink0} ${styles.wFull}`} style={{ height: "20px" }} data-name="Heading 3">
      <p className={`${styles.absolute} ${styles.fontNormal} ${styles.leading20} ${styles.notItalic} ${styles.text14} ${styles.textCenter} ${styles.textNowrap} ${styles.textWhite} ${styles.trackingNeg1504} ${styles.whitespacePre}`}
        style={{ left: "68.56px", top: "0.82px", transform: "translateX(-50%)" }}
      >Workspace Panel</p>
    </div>
  );
}

function Paragraph() {
  return (
    <div className={`${styles.contentStretch} ${styles.flex} ${styles.itemsStart} ${styles.relative} ${styles.shrink0} ${styles.wFull}`}
      style={{ height: "15.994px" }}
      data-name="Paragraph"
    >
      <p className={`${styles.fontNormal} ${styles.leading16} ${styles.notItalic} ${styles.relative} ${styles.shrink0} ${styles.textGray500} ${styles.text12} ${styles.textCenter} ${styles.textNowrap} ${styles.whitespacePre}`}>Content will appear here</p>
    </div>
  );
}

function Container2() {
  return (
    <div className={`${styles.absolute} ${styles.contentStretch} ${styles.flexCol} ${styles.itemsCenter}`}
      style={{ gap: "10px", left: "calc(50% + 0.055px)", top: "calc(50% - 0.818px)", transform: "translate(-50%, -50%)", width: "138.111px" }}
      data-name="Container"
    >
      <Container1 />
      <Heading1 />
      <Paragraph />
    </div>
  );
}

function Frame() {
  return (
    <div className={`${styles.basis0} ${styles.grow} ${styles.minHPx} ${styles.minWPx} ${styles.relative} ${styles.shrink0} ${styles.wFull}`}>
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
    <div className={`${styles.bgDarkPrimary} ${styles.contentStretch} ${styles.flexCol} ${styles.itemsCenter} ${styles.relative} ${styles.sizeFull}`}
      style={{ backgroundColor: "#0d0d0d", borderRadius: "16px" }}
      data-name="WorkspacePanel"
    >
      <div aria-hidden="true" className={`${styles.absolute} ${styles.borderSolid} ${styles.inset0} ${styles.pointerEventsNone}`}
        style={{ borderWidth: "0px 0.909px 0px 0px", borderColor: "#262626", borderRadius: "16px" }}
      />
      <Container title={title} onClose={onClose} />
      <Frame />
    </div>
  );
}