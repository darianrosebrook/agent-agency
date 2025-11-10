import { useState } from 'react';
import { Folder } from 'lucide-react';
import svgPaths from '../imports/svg-jzcqnicw4t';
import { ChatSidebar } from './ChatSidebar';
import WorkspacePanel from '../imports/WorkspacePanel';

export function WorkspaceTab() {
  const [activeTab, setActiveTab] = useState<'context' | 'chats'>('context');
  const [selectedItem, setSelectedItem] = useState<string | null>(null);
  const [isSidebarCollapsed, setIsSidebarCollapsed] = useState(false);

  return (
    <div className="h-full w-full overflow-hidden bg-[#0d0d0d]">
      <div className="h-full overflow-auto">
        <div className="box-border flex flex-col gap-[24px] h-full items-start pb-0 pt-[32px] px-[32px]">  

          {/* Main Content */}
          <div className="content-stretch flex gap-[16px] items-start w-full flex-1 min-h-0 relative">
            {/* Expand Button - shown when sidebar is collapsed */}
            {isSidebarCollapsed && (
              <button
                onClick={() => setIsSidebarCollapsed(false)}
                className="absolute left-0 top-0 z-10 bg-[#1a1a1a] border border-neutral-800 flex items-center justify-center rounded-full size-[40px] hover:bg-[#252525] transition-colors shadow-lg"
              >
                <div className="size-[16px]">
                  <svg className="block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 16 16">
                    <path d={svgPaths.p24b5a500} stroke="#888888" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
                  </svg>
                </div>
              </button>
            )}
            
            {/* Workspace Sidebar */}
            <div className={`bg-[#1a1a1a] flex flex-col h-full rounded-[16px] shrink-0 overflow-hidden transition-all duration-300 ${
              isSidebarCollapsed ? 'w-0 opacity-0 -ml-[16px]' : 'w-[319px] opacity-100'
            }`}>
              <div aria-hidden="true" className="absolute border-[#1e2939] border-[0px_0.909px_0px_0px] border-solid inset-0 pointer-events-none rounded-[16px]" />
              
              {/* Header */}
              <div className="relative shrink-0 w-full">
                <div aria-hidden="true" className="absolute border-[#1e2939] border-[0px_0px_0.909px] border-solid inset-0 pointer-events-none" />
                <div className="flex flex-col h-[60px] items-start pb-[0.909px] pt-[16px] px-[16px]">
                  <div className="h-[27px] relative w-full flex items-center">
                    <div className="absolute left-0 size-[16px] top-[6px]">
                      <svg className="block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 16 16">
                        <g clipPath="url(#clip0_16_2398)">
                          <path d={svgPaths.p14b1d380} stroke="#FECA57" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
                        </g>
                        <defs>
                          <clipPath id="clip0_16_2398">
                            <rect fill="white" height="15.9943" width="15.9943" />
                          </clipPath>
                        </defs>
                      </svg>
                    </div>
                    <p className="font-['Inter:Medium',sans-serif] font-medium leading-[27px] ml-[28px] text-[18px] text-white tracking-[-0.4395px]">
                      Workspace
                    </p>
                    <button 
                      onClick={() => setIsSidebarCollapsed(!isSidebarCollapsed)}
                      className="absolute bg-[#1a1a1a] border border-neutral-800 flex items-center justify-center right-0 rounded-full size-[32px] top-[-2px] hover:bg-[#252525] transition-colors"
                    >
                      <div className="size-[16px]">
                        <svg className="block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 16 16">
                          <path d={svgPaths.pc477740} stroke="#888888" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
                        </svg>
                      </div>
                    </button>
                  </div>
                </div>
              </div>

              {/* Tab List */}
              <div className="bg-[#0d0d0d] h-[40px] relative shrink-0 w-full">
                <div aria-hidden="true" className="absolute border-[0px_0px_0.909px] border-neutral-800 border-solid inset-0 pointer-events-none" />
                <div className="h-[40px] relative w-full flex items-center px-[3px]">
                  <button
                    onClick={() => setActiveTab('context')}
                    className={`flex gap-[6px] h-[32px] items-center justify-center px-[9px] py-[5px] rounded-[14px] w-[50%] transition-colors ${
                      activeTab === 'context' ? 'bg-[#1a1a1a]' : 'hover:bg-[#1a1a1a]/50'
                    }`}
                  >
                    <p className={`font-['Inter:Medium',sans-serif] font-medium leading-[20px] text-[14px] tracking-[-0.1504px] ${
                      activeTab === 'context' ? 'text-white' : 'text-[#888888]'
                    }`}>
                      Context
                    </p>
                  </button>
                  <button
                    onClick={() => setActiveTab('chats')}
                    className={`flex gap-[6px] h-[32px] items-center justify-center px-[9px] py-[5px] rounded-[14px] w-[50%] transition-colors ${
                      activeTab === 'chats' ? 'bg-[#1a1a1a]' : 'hover:bg-[#1a1a1a]/50'
                    }`}
                  >
                    <p className={`font-['Inter:Medium',sans-serif] font-medium leading-[20px] text-[14px] tracking-[-0.1504px] ${
                      activeTab === 'chats' ? 'text-white' : 'text-[#888888]'
                    }`}>
                      Chats
                    </p>
                  </button>
                </div>
              </div>

              {/* Tab Content */}
              <div className="flex-1 overflow-auto">
                {activeTab === 'context' ? (
                  <ContextFileTree onSelect={setSelectedItem} />
                ) : (
                  <div className="h-full">
                    <ChatSidebar onSelect={setSelectedItem} />
                  </div>
                )}
              </div>
            </div>
            
            {/* Workspace Panel - appears when item is selected */}
            {selectedItem && (
              <div className="flex-[2] min-h-0 h-full">
                <WorkspacePanel title={selectedItem} onClose={() => setSelectedItem(null)} />
              </div>
            )}

            {/* Bento Grid - reflows based on sidebar and panel visibility */}
            <div className={`min-h-0 transition-all duration-300 ${selectedItem ? 'flex-1' : 'flex-[2]'}`}>
              <BentoGrid hasPanel={!!selectedItem} />
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

interface ContextFileTreeProps {
  onSelect: (item: string) => void;
}

function ContextFileTree({ onSelect }: ContextFileTreeProps) {
  const [expandedFolders, setExpandedFolders] = useState<Set<string>>(new Set(['src', 'public']));

  const toggleFolder = (folderId: string) => {
    const newExpanded = new Set(expandedFolders);
    if (newExpanded.has(folderId)) {
      newExpanded.delete(folderId);
    } else {
      newExpanded.add(folderId);
    }
    setExpandedFolders(newExpanded);
  };

  return (
    <div className="w-full p-[8px]">
      {/* src folder */}
      <div className="mb-0">
        <button
          onClick={() => toggleFolder('src')}
          className="flex gap-[8px] h-[32px] items-center pl-[8px] pr-0 py-0 rounded-[4px] w-full hover:bg-[#252525] transition-colors"
        >
          <div className="size-[16px] shrink-0">
            <svg className="block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 16 16">
              <path d={expandedFolders.has('src') ? svgPaths.p10a02b40 : svgPaths.p24b5a500} stroke="#D1D5DC" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
            </svg>
          </div>
          <div className="size-[16px] shrink-0">
            <svg className="block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 16 16">
              <g clipPath="url(#clip0_15_2243)">
                <path d={svgPaths.p14b1d380} stroke="#FECA57" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
              </g>
              <defs>
                <clipPath id="clip0_15_2243">
                  <rect fill="white" height="15.9943" width="15.9943" />
                </clipPath>
              </defs>
            </svg>
          </div>
          <p className="font-['Inter:Regular',sans-serif] font-normal leading-[20px] text-[#d1d5dc] text-[14px] tracking-[-0.1504px]">
            src
          </p>
        </button>
        {expandedFolders.has('src') && (
          <div className="flex flex-col gap-0">
            <button onClick={() => onSelect('src/components')} className="flex gap-[8px] h-[32px] items-center pl-[20px] pr-0 py-0 rounded-[4px] w-full hover:bg-[#252525] transition-colors">
              <div className="size-[16px] shrink-0">
                <svg className="block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 16 16">
                  <path d={svgPaths.p24b5a500} stroke="#D1D5DC" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
                </svg>
              </div>
              <div className="size-[16px] shrink-0">
                <svg className="block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 16 16">
                  <g clipPath="url(#clip0_15_2228)">
                    <path d={svgPaths.p8e3b480} stroke="#FECA57" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
                  </g>
                  <defs>
                    <clipPath id="clip0_15_2228">
                      <rect fill="white" height="15.9943" width="15.9943" />
                    </clipPath>
                  </defs>
                </svg>
              </div>
              <p className="font-['Inter:Regular',sans-serif] font-normal leading-[20px] text-[#d1d5dc] text-[14px] tracking-[-0.1504px]">
                components
              </p>
            </button>
            <button onClick={() => onSelect('src/utils')} className="flex gap-[8px] h-[32px] items-center pl-[20px] pr-0 py-0 rounded-[4px] w-full hover:bg-[#252525] transition-colors">
              <div className="size-[16px] shrink-0">
                <svg className="block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 16 16">
                  <path d={svgPaths.p24b5a500} stroke="#D1D5DC" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
                </svg>
              </div>
              <div className="size-[16px] shrink-0">
                <svg className="block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 16 16">
                  <g clipPath="url(#clip0_15_2228)">
                    <path d={svgPaths.p8e3b480} stroke="#FECA57" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
                  </g>
                  <defs>
                    <clipPath id="clip0_15_2228">
                      <rect fill="white" height="15.9943" width="15.9943" />
                    </clipPath>
                  </defs>
                </svg>
              </div>
              <p className="font-['Inter:Regular',sans-serif] font-normal leading-[20px] text-[#d1d5dc] text-[14px] tracking-[-0.1504px]">
                utils
              </p>
            </button>
          </div>
        )}
      </div>

      {/* public folder */}
      <div className="mb-0">
        <button
          onClick={() => toggleFolder('public')}
          className="flex gap-[8px] h-[32px] items-center pl-[8px] pr-0 py-0 rounded-[4px] w-full hover:bg-[#252525] transition-colors"
        >
          <div className="size-[16px] shrink-0">
            <svg className="block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 16 16">
              <path d={expandedFolders.has('public') ? svgPaths.p10a02b40 : svgPaths.p24b5a500} stroke="#D1D5DC" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
            </svg>
          </div>
          <div className="size-[16px] shrink-0">
            <svg className="block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 16 16">
              <g clipPath="url(#clip0_15_2243)">
                <path d={svgPaths.p14b1d380} stroke="#FECA57" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
              </g>
              <defs>
                <clipPath id="clip0_15_2243">
                  <rect fill="white" height="15.9943" width="15.9943" />
                </clipPath>
              </defs>
            </svg>
          </div>
          <p className="font-['Inter:Regular',sans-serif] font-normal leading-[20px] text-[#d1d5dc] text-[14px] tracking-[-0.1504px]">
            public
          </p>
        </button>
        {expandedFolders.has('public') && (
          <div className="flex flex-col gap-0">
            <button onClick={() => onSelect('public/index.html')} className="flex gap-[8px] h-[32px] items-center pl-[36px] pr-0 py-0 rounded-[4px] w-full hover:bg-[#252525] transition-colors">
              <div className="size-[16px] shrink-0">
                <svg className="block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 16 16">
                  <g clipPath="url(#clip0_15_2221)">
                    <path d={svgPaths.p1aaaa600} stroke="#888888" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
                    <path d={svgPaths.p1bffbec0} stroke="#888888" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
                    <path d="M6.6643 5.99787H5.33144" stroke="#888888" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
                    <path d="M10.6629 8.66359H5.33144" stroke="#888888" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
                    <path d="M10.6629 11.3293H5.33144" stroke="#888888" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
                  </g>
                  <defs>
                    <clipPath id="clip0_15_2221">
                      <rect fill="white" height="15.9943" width="15.9943" />
                    </clipPath>
                  </defs>
                </svg>
              </div>
              <p className="font-['Inter:Regular',sans-serif] font-normal leading-[20px] text-[#d1d5dc] text-[14px] tracking-[-0.1504px]">
                index.html
              </p>
            </button>
            <button onClick={() => onSelect('public/styles.css')} className="flex gap-[8px] h-[32px] items-center pl-[36px] pr-0 py-0 rounded-[4px] w-full hover:bg-[#252525] transition-colors">
              <div className="size-[16px] shrink-0">
                <svg className="block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 16 16">
                  <g clipPath="url(#clip0_15_2221)">
                    <path d={svgPaths.p1aaaa600} stroke="#888888" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
                    <path d={svgPaths.p1bffbec0} stroke="#888888" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
                    <path d="M6.6643 5.99787H5.33144" stroke="#888888" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
                    <path d="M10.6629 8.66359H5.33144" stroke="#888888" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
                    <path d="M10.6629 11.3293H5.33144" stroke="#888888" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
                  </g>
                  <defs>
                    <clipPath id="clip0_15_2221">
                      <rect fill="white" height="15.9943" width="15.9943" />
                    </clipPath>
                  </defs>
                </svg>
              </div>
              <p className="font-['Inter:Regular',sans-serif] font-normal leading-[20px] text-[#d1d5dc] text-[14px] tracking-[-0.1504px]">
                styles.css
              </p>
            </button>
          </div>
        )}
      </div>

      {/* package.json */}
      <button onClick={() => onSelect('package.json')} className="flex gap-[8px] h-[32px] items-center pl-[24px] pr-0 py-0 rounded-[4px] w-full hover:bg-[#252525] transition-colors">
        <div className="size-[16px] shrink-0">
          <svg className="block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 16 16">
            <g clipPath="url(#clip0_15_2231)">
              <path d={svgPaths.p1aaaa600} stroke="#FECA57" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
              <path d={svgPaths.p1bffbec0} stroke="#FECA57" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
              <path d={svgPaths.p89bb2c0} stroke="#FECA57" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
              <path d={svgPaths.p381b1faa} stroke="#FECA57" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
            </g>
            <defs>
              <clipPath id="clip0_15_2231">
                <rect fill="white" height="15.9943" width="15.9943" />
              </clipPath>
            </defs>
          </svg>
        </div>
        <p className="font-['Inter:Regular',sans-serif] font-normal leading-[20px] text-[#d1d5dc] text-[14px] tracking-[-0.1504px]">
          package.json
        </p>
      </button>

      {/* README.md */}
      <button onClick={() => onSelect('README.md')} className="flex gap-[8px] h-[32px] items-center pl-[24px] pr-0 py-0 rounded-[4px] w-full hover:bg-[#252525] transition-colors">
        <div className="size-[16px] shrink-0">
          <svg className="block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 16 16">
            <g clipPath="url(#clip0_15_2221)">
              <path d={svgPaths.p1aaaa600} stroke="#888888" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
              <path d={svgPaths.p1bffbec0} stroke="#888888" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
              <path d="M6.6643 5.99787H5.33144" stroke="#888888" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
              <path d="M10.6629 8.66359H5.33144" stroke="#888888" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
              <path d="M10.6629 11.3293H5.33144" stroke="#888888" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.33286" />
            </g>
            <defs>
              <clipPath id="clip0_15_2221">
                <rect fill="white" height="15.9943" width="15.9943" />
              </clipPath>
            </defs>
          </svg>
        </div>
        <p className="font-['Inter:Regular',sans-serif] font-normal leading-[20px] text-[#d1d5dc] text-[14px] tracking-[-0.1504px]">
          README.md
        </p>
      </button>
    </div>
  );
}

interface BentoGridProps {
  hasPanel: boolean;
}

function BentoGrid({ hasPanel }: BentoGridProps) {
  return (
    <div className={`grid ${hasPanel ? 'grid-cols-1' : 'grid-cols-12'} gap-4 h-full auto-rows-[minmax(200px,auto)]`}>
      {/* Panel 1 - spans 4 columns, 2 rows */}
      <div className={`${hasPanel ? 'col-span-1' : 'col-span-4'} row-span-2 bg-[#1a1a1a] border border-[#1e2939] rounded-[16px] p-4 flex items-center justify-center`}>
        <p className="text-[#4a5565] text-[16px]">Panel 1</p>
      </div>

      {/* Panel 2 - spans 8 columns, 2 rows */}
      <div className={`${hasPanel ? 'col-span-1' : 'col-span-8'} row-span-2 bg-[#1a1a1a] border border-[#1e2939] rounded-[16px] p-4 flex items-center justify-center`}>
        <p className="text-[#4a5565] text-[16px]">Panel 2</p>
      </div>

      {/* Panel 3 - spans 7 columns, 3 rows */}
      <div className={`${hasPanel ? 'col-span-1' : 'col-span-7'} row-span-3 bg-[#1a1a1a] border border-[#1e2939] rounded-[16px] p-4 flex items-center justify-center`}>
        <p className="text-[#4a5565] text-[16px]">Panel 3</p>
      </div>

      {/* Panel 4 - spans 5 columns, 3 rows */}
      <div className={`${hasPanel ? 'col-span-1' : 'col-span-5'} row-span-3 bg-[#1a1a1a] border border-[#1e2939] rounded-[16px] p-4 flex items-center justify-center`}>
        <p className="text-[#4a5565] text-[16px]">Panel 4</p>
      </div>

      {/* Panel 5 - spans 12 columns, 3 rows */}
      <div className={`${hasPanel ? 'col-span-1' : 'col-span-12'} row-span-3 bg-[#1a1a1a] border border-[#1e2939] rounded-[16px] p-4 flex items-center justify-center`}>
        <p className="text-[#4a5565] text-[16px]">Panel 5</p>
      </div>

      {/* Panel 6 - spans 4 columns, 2 rows */}
      <div className={`${hasPanel ? 'col-span-1' : 'col-span-4'} row-span-2 bg-[#1a1a1a] border border-[#1e2939] rounded-[16px] p-4 flex items-center justify-center`}>
        <p className="text-[#4a5565] text-[16px]">Panel 6</p>
      </div>

      {/* Panel 7 - spans 4 columns, 2 rows */}
      <div className={`${hasPanel ? 'col-span-1' : 'col-span-4'} row-span-2 bg-[#1a1a1a] border border-[#1e2939] rounded-[16px] p-4 flex items-center justify-center`}>
        <p className="text-[#4a5565] text-[16px]">Panel 7</p>
      </div>

      {/* Panel 8 - spans 4 columns, 2 rows */}
      <div className={`${hasPanel ? 'col-span-1' : 'col-span-4'} row-span-2 bg-[#1a1a1a] border border-[#1e2939] rounded-[16px] p-4 flex items-center justify-center`}>
        <p className="text-[#4a5565] text-[16px]">Panel 8</p>
      </div>
    </div>
  );
}
