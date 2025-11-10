import styles from "./Frame344923414.module.scss";

function Frame1() {
  return (
    <div className={`${styles.contentStretch} ${styles.flex} ${styles.gap8px} ${styles.itemsStart} ${styles.relative} ${styles.shrink0}`}>
      <ComponentStatusBadge />
      <ComponentStatusBadge />
    </div>
  );
}

function Frame2() {
  return (
    <div className={`${styles.contentStretch} ${styles.flex} ${styles.itemsStart} ${styles.justifyBetween} ${styles.relative} ${styles.shrink0} ${styles.wFull}`}>
      <Frame1 />
      <ComponentStatusBadge />
    </div>
  );
}

function Frame3() {
  return (
    <div className={`${styles.contentStretch} ${styles.flex} ${styles.flexCol} ${styles.gap8px} ${styles.itemsStart} ${styles.relative} ${styles.shrink0} ${styles.wFull}`}>
      <Frame2 />
      <div className={`${styles.flex} ${styles.flexCol} ${styles.fontMedium} ${styles.justifyCenter} ${styles.leadingNone} ${styles.notItalic} ${styles.relative} ${styles.shrink0} ${styles.text24px} ${styles.textCenter} ${styles.textNeutral50} ${styles.textNowrap} ${styles.trackingNeg24px}`}>
        <p className={`${styles.leadingNormal} ${styles.whitespacePre}`}>Task completion</p>
      </div>
    </div>
  );
}

function ComponentStatusBadge() {
  return (
    <div className={`${styles.relative} ${styles.rounded2px} ${styles.shrink0}`} data-name=".component status badge">
      <div className={`${styles.boxBorder} ${styles.contentStretch} ${styles.flex} ${styles.itemsCenter} ${styles.overflowClip} ${styles.px4px} ${styles.py2px} ${styles.relative} ${styles.roundedInherit}`}>
        <div className={`${styles.w2} ${styles.h2} ${styles.bgCacaca} ${styles.rounded}`} />
        <div className={`${styles.flex} ${styles.flexCol} ${styles.fontBold} ${styles.justifyCenter} ${styles.leadingNone} ${styles.notItalic} ${styles.relative} ${styles.shrink0} ${styles.textCacaca} ${styles.text8px} ${styles.textNowrap} ${styles.textRight} ${styles.tracking012px}`}>
          <p className={`${styles.leadingNone} ${styles.whitespacePre}`}>12 tasks</p>
        </div>
      </div>
      <div aria-hidden="true" className={`${styles.absolute} ${styles.border} ${styles.borderCacaca} ${styles.borderSolid} ${styles.inset0} ${styles.pointerEventsNone} ${styles.rounded2px}`} />
    </div>
  );
}

function Frame5() {
  return (
    <div className={`${styles.contentStretch} ${styles.flex} ${styles.gap8px} ${styles.itemsCenter} ${styles.relative} ${styles.shrink0}`}>
      <div className={`${styles.flex} ${styles.flexCol} ${styles.fontLight} ${styles.justifyEnd} ${styles.leadingNone} ${styles.notItalic} ${styles.relative} ${styles.shrink0} ${styles.text48px} ${styles.textCenter} ${styles.textNeutral50} ${styles.textNowrap} ${styles.trackingNeg24px}`}>
        <p className={`${styles.leading48px} ${styles.whitespacePre}`}>71%</p>
      </div>
      <ComponentStatusBadge />
      <div className={`${styles.flex} ${styles.flexCol} ${styles.fontMedium} ${styles.justifyCenter} ${styles.leadingNone} ${styles.notItalic} ${styles.relative} ${styles.shrink0} ${styles.textCacaca} ${styles.text10px} ${styles.textCenter} ${styles.textNowrap} ${styles.trackingNeg01px}`}>
        <p className={`${styles.leading12px} ${styles.whitespacePre}`}>since last week</p>
      </div>
    </div>
  );
}

function Frame4() {
  return (
    <div className={`${styles.contentStretch} ${styles.flex} ${styles.flexCol} ${styles.gap8px} ${styles.itemsStart} ${styles.relative} ${styles.shrink0}`}>
      <div className={`${styles.flex} ${styles.flexCol} ${styles.fontMedium} ${styles.justifyCenter} ${styles.leadingNone} ${styles.notItalic} ${styles.relative} ${styles.shrink0} ${styles.textCacaca} ${styles.text10px} ${styles.textCenter} ${styles.textNowrap} ${styles.trackingNeg01px}`}>
        <p className={`${styles.leading12px} ${styles.whitespacePre}`}>You have 12 tasks out of 30 completed</p>
      </div>
      <Frame5 />
    </div>
  );
}

function Frame() {
  const barClass = `${styles.basis0} ${styles.grow} ${styles.h108px} ${styles.minHPx} ${styles.minWPx} ${styles.shrink0}`;
  return (
    <div className={`${styles.basis0} ${styles.contentStretch} ${styles.flex} ${styles.gap2px} ${styles.grow} ${styles.itemsCenter} ${styles.maxH64px} ${styles.minHPx} ${styles.minWPx} ${styles.overflowClip} ${styles.relative} ${styles.rounded4px} ${styles.shrink0} ${styles.wFull}`}>
      <div className={`${barClass} ${styles.bgNeutral50}`} />
      <div className={`${barClass} ${styles.bgNeutral50}`} />
      <div className={`${barClass} ${styles.bgNeutral50}`} />
      <div className={`${barClass} ${styles.bgNeutral50}`} />
      <div className={`${barClass} ${styles.bgNeutral50}`} />
      <div className={`${barClass} ${styles.bgNeutral50}`} />
      <div className={`${barClass} ${styles.bgNeutral50}`} />
      <div className={`${barClass} ${styles.bgNeutral50}`} />
      <div className={`${barClass} ${styles.bgNeutral50}`} />
      <div className={`${barClass} ${styles.bgNeutral50}`} />
      <div className={`${barClass} ${styles.bgNeutral50}`} />
      <div className={`${barClass} ${styles.bgNeutral50}`} />
      <div className={`${barClass} ${styles.bgNeutral50}`} />
      <div className={`${barClass} ${styles.bgNeutral50}`} />
      <div className={`${barClass} ${styles.bgNeutral50}`} />
      <div className={`${barClass} ${styles.bgNeutral50}`} />
      <div className={`${barClass} ${styles.bgNeutral50}`} />
      <div className={`${barClass} ${styles.bgNeutral50}`} />
      <div className={`${barClass} ${styles.bgNeutral50}`} />
      <div className={`${barClass} ${styles.bg454545}`} />
      <div className={`${barClass} ${styles.bg454545}`} />
      <div className={`${barClass} ${styles.bg454545}`} />
      <div className={`${barClass} ${styles.bg454545}`} />
      <div className={`${barClass} ${styles.bg454545}`} />
      <div className={`${barClass} ${styles.bg454545}`} />
      <div className={`${barClass} ${styles.bg454545}`} />
      <div className={`${barClass} ${styles.bg454545}`} />
      <div className={`${barClass} ${styles.bg454545}`} />
      <div className={`${barClass} ${styles.bg454545}`} />
      <div className={`${barClass} ${styles.bg454545}`} />
      <div className={`${barClass} ${styles.bg454545}`} />
      <div className={`${barClass} ${styles.bg454545}`} />
      <div className={`${barClass} ${styles.bg454545}`} />
      <div className={`${barClass} ${styles.bg454545}`} />
      <div className={`${barClass} ${styles.bg454545}`} />
      <div className={`${barClass} ${styles.bg454545}`} />
      <div className={`${barClass} ${styles.bg454545}`} />
      <div className={`${barClass} ${styles.bg454545}`} />
      <div className={`${barClass} ${styles.bg454545}`} />
      <div className={`${barClass} ${styles.bg454545}`} />
    </div>
  );
}

export default function Frame6() {
  return (
    <div className={`${styles.bgNeutral950} ${styles.relative} ${styles.rounded12px} ${styles.sizeFull}`}>
      <div className={styles.sizeFull}>
        <div className={`${styles.boxBorder} ${styles.contentStretch} ${styles.flex} ${styles.flexCol} ${styles.gap16px} ${styles.itemsStart} ${styles.overflowClip} ${styles.p12px} ${styles.relative} ${styles.sizeFull}`}>
          <Frame3 />
          <div className={`${styles.bgCacaca} ${styles.hPx} ${styles.shrink0} ${styles.wFull}`} />
          <Frame4 />
          <Frame />
        </div>
      </div>
      <div aria-hidden="true" className={`${styles.absolute} ${styles.border} ${styles.borderCacaca} ${styles.borderSolid} ${styles.inset0} ${styles.pointerEventsNone} ${styles.rounded12px}`} />
    </div>
  );
}