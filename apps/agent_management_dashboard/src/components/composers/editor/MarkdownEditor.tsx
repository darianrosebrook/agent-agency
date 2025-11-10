"use client";

import type { ReactNode } from "react";
import { cn } from "../../primitives/utils";
import {
  MarkdownHeading,
  MarkdownParagraph,
  MarkdownQuote,
  MarkdownListItem,
  MarkdownImagePlaceholder,
  MarkdownCodeBlock,
  MarkdownBold,
  MarkdownItalic,
  MarkdownLink,
  MarkdownInlineCode,
} from "../../compounds/editor";
import imgImagePlaceholderImage from "@/assets/8fc47397db8ccbdd80313630807c1d1d61a2924b.png";
import styles from "./MarkdownEditor.module.scss";

export interface MarkdownContent {
  headings?: Array<{ level?: 1 | 2 | 3 | 4 | 5 | 6; text: ReactNode; style?: React.CSSProperties }>;
  paragraphs?: Array<{ text: ReactNode; style?: React.CSSProperties }>;
  quotes?: Array<{ text: ReactNode; style?: React.CSSProperties }>;
  listItems?: Array<{ text: ReactNode; style?: React.CSSProperties }>;
  images?: Array<{ src: string; alt?: string; style?: React.CSSProperties }>;
  codeBlocks?: Array<{ code: ReactNode; style?: React.CSSProperties }>;
}

interface MarkdownEditorProps {
  content?: MarkdownContent;
  className?: string;
}

const defaultContent: MarkdownContent = {
  headings: [
    { level: 1, text: "Getting started" },
    { level: 2, text: "Features", style: { top: "319.66px" } },
    { level: 2, text: "Make it your own", style: { top: "1065.43px" } },
  ],
  paragraphs: [
    {
      text: (
        <>
          Welcome to the <MarkdownItalic>Simple Editor</MarkdownItalic> template! This is an{" "}
          <MarkdownBold>MIT</MarkdownBold> licensed, <MarkdownBold>open source</MarkdownBold> template.
        </>
      ),
    },
    {
      text: (
        <>
          Integrate it by following the <MarkdownLink href="#">Tiptap UI Components docs</MarkdownLink> or
          using our CLI tool.
        </>
      ),
      style: { left: 0, top: "138.91px", width: "800px" },
    },
    {
      text: (
        <>
          Add images, customize alignment, and apply <MarkdownBold>advanced formatting</MarkdownBold> to
          make your writing more engaging and professional.
        </>
      ),
      style: { left: 0, top: "487.6px", width: "800px" },
    },
    {
      text: (
        <>
          → <MarkdownLink href="#">Learn more</MarkdownLink>
        </>
      ),
      style: { left: 0, top: "995.47px", width: "800px" },
    },
    {
      text: "Switch between light and dark modes, and tailor the editor's appearance with customizable CSS to match your style.",
      style: { height: "51.96px", left: 0, top: "1129.4px", width: "800px" },
    },
  ],
  quotes: [
    {
      text: (
        <>
          A fully responsive rich text editor with built-in support for common formatting and layout tools.
          Type markdown <MarkdownInlineCode>++</MarkdownInlineCode> or use keyboard shortcuts{" "}
          <MarkdownInlineCode>⌘B</MarkdownInlineCode> for most all common markdown marks. ✨
        </>
      ),
    },
  ],
  listItems: [
    {
      text: (
        <>
          <MarkdownBold>Superscript</MarkdownBold> (x²) and <MarkdownBold>Subscript</MarkdownBold> (H₂O)
          for precision.
        </>
      ),
    },
    {
      text: (
        <>
          <MarkdownBold>Typographic conversion</MarkdownBold>: automatically convert to -&gt; an arrow →
        </>
      ),
      style: { top: "947.5px" },
    },
  ],
  codeBlocks: [
    {
      code: "npx @tiptap/cli init",
    },
  ],
  images: [
    {
      src: typeof imgImagePlaceholderImage === "string" ? imgImagePlaceholderImage : imgImagePlaceholderImage.src,
      alt: "",
    },
  ],
};

export function MarkdownEditor({ content, className = "" }: MarkdownEditorProps) {
  // Use provided content or default content
  const displayContent = content || defaultContent;

  return (
    <div className={cn(styles.markdownEditor, className)} data-name="MarkdownEditor">
      {/* Render in original order: Heading, Paragraph, Paragraph1, CodeBlock, Heading1, Quote, Paragraph2, Image, ListItems, Paragraph3, Heading3, Paragraph4 */}
      {displayContent.headings?.[0] && (
        <MarkdownHeading level={displayContent.headings[0].level} style={displayContent.headings[0].style}>
          {displayContent.headings[0].text}
        </MarkdownHeading>
      )}
      {displayContent.paragraphs?.[0] && (
        <MarkdownParagraph style={displayContent.paragraphs[0].style}>
          {displayContent.paragraphs[0].text}
        </MarkdownParagraph>
      )}
      {displayContent.paragraphs?.[1] && (
        <MarkdownParagraph style={displayContent.paragraphs[1].style}>
          {displayContent.paragraphs[1].text}
        </MarkdownParagraph>
      )}
      {displayContent.codeBlocks?.[0] && (
        <MarkdownCodeBlock style={displayContent.codeBlocks[0].style}>
          {displayContent.codeBlocks[0].code}
        </MarkdownCodeBlock>
      )}
      {displayContent.headings?.[1] && (
        <MarkdownHeading level={displayContent.headings[1].level} style={displayContent.headings[1].style}>
          {displayContent.headings[1].text}
        </MarkdownHeading>
      )}
      {displayContent.quotes?.[0] && (
        <MarkdownQuote style={displayContent.quotes[0].style}>
          {displayContent.quotes[0].text}
        </MarkdownQuote>
      )}
      {displayContent.paragraphs?.[2] && (
        <MarkdownParagraph style={displayContent.paragraphs[2].style}>
          {displayContent.paragraphs[2].text}
        </MarkdownParagraph>
      )}
      {displayContent.images?.[0] && (
        <MarkdownImagePlaceholder
          src={displayContent.images[0].src}
          alt={displayContent.images[0].alt}
          style={displayContent.images[0].style}
        />
      )}
      {displayContent.listItems?.map((item, index) => (
        <MarkdownListItem key={index} style={item.style}>
          {item.text}
        </MarkdownListItem>
      ))}
      {displayContent.paragraphs?.[3] && (
        <MarkdownParagraph style={displayContent.paragraphs[3].style}>
          {displayContent.paragraphs[3].text}
        </MarkdownParagraph>
      )}
      {displayContent.headings?.[2] && (
        <MarkdownHeading level={displayContent.headings[2].level} style={displayContent.headings[2].style}>
          {displayContent.headings[2].text}
        </MarkdownHeading>
      )}
      {displayContent.paragraphs?.[4] && (
        <MarkdownParagraph style={displayContent.paragraphs[4].style}>
          {displayContent.paragraphs[4].text}
        </MarkdownParagraph>
      )}
    </div>
  );
}

