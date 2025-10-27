import React from "react";

type Props = {
  Header?: React.ReactNode;
  ContextTray?: React.ReactNode;
  InputArea: React.ReactNode;        // required
  FooterLeft?: React.ReactNode;
  FooterRight?: React.ReactNode;
};

export function Composer({ Header, ContextTray, InputArea, FooterLeft, FooterRight }: Props) {
  return (
    <div className="w-full max-w-3xl space-y-6">
      {Header}
      {ContextTray}
      <div className="relative rounded-xl border-2 bg-input border-border hover:border-ring/50 transition-all">
        {InputArea}
        <div className="absolute bottom-0 left-0 right-0 flex items-center justify-between px-3 py-3 border-t border-border/50">
          <div className="flex items-center gap-1">{FooterLeft}</div>
          <div className="flex items-center gap-1">{FooterRight}</div>
        </div>
      </div>
    </div>
  );
}
