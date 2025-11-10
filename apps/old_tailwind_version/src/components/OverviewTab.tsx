import { useState } from 'react';
import { PanelRightOpen, PanelRightClose } from 'lucide-react';
import OverviewEditorImport from '../imports/Container';
import svgPaths from '../imports/svg-8d8l4g1ml9';
import { useProjectContext } from './ProjectContext';

export function OverviewTab() {
  const [showMetadata, setShowMetadata] = useState(true);
  const { getCurrentProject } = useProjectContext();
  const currentProject = getCurrentProject();

  return (
    <div className="relative h-full w-full">
      {/* Toggle Button */}
      <button
        onClick={() => setShowMetadata(!showMetadata)}
        className="absolute top-4 right-4 z-10 bg-[#1a1a1a] hover:bg-[#252525] text-gray-300 hover:text-white rounded-lg p-2 transition-colors border border-gray-800"
        title={showMetadata ? 'Hide metadata panel' : 'Show metadata panel'}
      >
        {showMetadata ? (
          <PanelRightClose className="w-5 h-5" />
        ) : (
          <PanelRightOpen className="w-5 h-5" />
        )}
      </button>

      {/* Editor Content */}
      <div className="h-full w-full overflow-hidden">
        {showMetadata ? (
          <OverviewEditorImport />
        ) : (
          <EditorOnly description={currentProject?.description} />
        )}
      </div>
    </div>
  );
}

// Editor without metadata panel
function EditorOnly({ description }: { description?: string }) {
  return (
    <div className="bg-[#0d0d0d] h-full w-full">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border h-full overflow-clip relative rounded-[inherit] w-full">
        {/* Editor Toolbar */}
        <div className="absolute bg-[#0d0d0d] box-border content-stretch flex flex-col h-[52.898px] items-start left-0 pb-[0.909px] pt-[7.997px] px-[15.994px] top-0 w-full z-10">
          <div aria-hidden="true" className="absolute border-[0px_0px_0.909px] border-neutral-800 border-solid inset-0 pointer-events-none" />
          <EditorToolbar />
        </div>

        {/* Editor Content Area */}
        <div className="absolute box-border content-stretch flex flex-col items-start left-0 overflow-clip pb-0 pt-[63.977px] top-[52.9px] w-full h-[calc(100%-52.9px)]">
          <div className="w-full max-w-[800px] mx-auto px-8">
            <MarkdownEditorPlaceholder description={description} />
          </div>
        </div>
      </div>
    </div>
  );
}

function EditorToolbar() {
  return (
    <div className="h-[35.994px] relative shrink-0 w-full">
      {/* Bold Button */}
      <button className="absolute content-stretch flex items-center justify-center left-0 rounded-[8px] size-[31.989px] top-[2px] hover:bg-[#1a1a1a] transition-colors">
        <div className="relative shrink-0 size-[15.994px]">
          <svg className="block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 16 16">
            <path d={svgPaths.p1b11cb00} stroke="#888888" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
            <path d={svgPaths.p8cc4400} stroke="#888888" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
          </svg>
        </div>
      </button>

      {/* Italic Button */}
      <button className="absolute content-stretch flex items-center justify-center left-[35.98px] rounded-[8px] size-[31.989px] top-[2px] hover:bg-[#1a1a1a] transition-colors">
        <div className="relative shrink-0 size-[15.994px]">
          <svg className="block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 16 16">
            <path d={svgPaths.p271f800} stroke="#888888" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
            <path d={svgPaths.p7307940} stroke="#888888" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
          </svg>
        </div>
      </button>

      {/* Divider */}
      <div className="absolute bg-neutral-800 h-0 left-[75.95px] top-[18px] w-[0.994px]" />

      {/* Text Style Dropdown */}
      <button className="absolute box-border content-stretch flex h-[35.994px] items-center justify-between left-[84.93px] px-[11.989px] py-0 rounded-[8px] top-0 w-[95.994px] hover:bg-[#1a1a1a] transition-colors">
        <span className="font-['Inter:Regular',sans-serif] font-normal leading-[20px] not-italic text-white text-[14px] tracking-[-0.1504px]">Text</span>
        <div className="relative shrink-0 size-[15.994px]">
          <svg className="block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 16 16">
            <path d={svgPaths.p10a02b40} stroke="#717182" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" opacity="0.5" />
          </svg>
        </div>
      </button>

      {/* More toolbar buttons would go here */}
      <div className="absolute left-[190px] top-[2px] text-[12px] text-gray-600">
        Rich text editor toolbar
      </div>
    </div>
  );
}

function MarkdownEditorPlaceholder({ description }: { description?: string }) {
  if (description) {
    return (
      <div className="py-8">
        <div className="mb-6">
          <p className="text-[16px] leading-[26px] text-[#e6e6e6] tracking-[-0.3125px] whitespace-pre-wrap">
            {description}
          </p>
        </div>
        
        {/* Edit hint */}
        <div className="text-gray-600 text-[14px] mt-12">
          Click to start editing...
        </div>
      </div>
    );
  }

  return (
    <div className="py-8">
      {/* Heading */}
      <div className="mb-8">
        <h1 className="text-[30px] leading-[36px] text-white tracking-[0.3955px]">
          Project Vision
        </h1>
      </div>

      {/* Paragraph */}
      <div className="mb-6">
        <p className="text-[16px] leading-[26px] text-[#e6e6e6] tracking-[-0.3125px]">
          Start writing your project vision here. This is where you define the goals, objectives, and overall direction for your project.
        </p>
      </div>

      {/* Another section */}
      <div className="mb-6">
        <h2 className="text-[24px] leading-[32px] text-white tracking-[0.0703px] mb-4">
          Key Objectives
        </h2>
        <p className="text-[16px] leading-[26px] text-[#e6e6e6] tracking-[-0.3125px]">
          Define what success looks like for this project. What are the main deliverables and milestones?
        </p>
      </div>

      {/* Placeholder for more content */}
      <div className="text-gray-600 text-[14px] mt-12">
        Click to start editing...
      </div>
    </div>
  );
}
