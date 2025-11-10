# Performance Bottleneck Analysis

## Issue: 44-second page load time for GET /

### Identified Bottlenecks

#### 1. **GSAP Library Loading** (6.2MB) ⚠️ CRITICAL
- **Location**: `src/components/dashboard/NavigationSidebar.tsx`
- **Issue**: GSAP loads twice on every page load (sidebar width + content opacity animations)
- **Impact**: High - Large library blocking initial render
- **Status**: ✅ Partially optimized (caching added, CSS fallback implemented)
- **Remaining Issue**: Still loads on mount even when animations aren't needed
- **Recommendation**: Defer GSAP loading until user interacts with sidebar

#### 2. **Recharts Library** (5.2MB) ⚠️ HIGH
- **Location**: `src/components/CodeContributionChart.tsx`, `src/components/ModelContributionStream.tsx`, `src/components/primitives/chart.tsx`
- **Issue**: Large charting library imported synchronously
- **Impact**: High - Even though charts are lazy-loaded, Recharts is still bundled
- **Status**: Charts are lazy-loaded but Recharts bundle is large
- **Recommendation**: 
  - Consider tree-shaking unused Recharts components
  - Use dynamic imports for Recharts itself
  - Consider lighter alternatives (Chart.js, Victory)

#### 3. **Zustand DevTools Middleware** ⚠️ MEDIUM
- **Location**: `src/lib/stores/projectStore.ts`, `src/lib/stores/chatStore.ts`
- **Issue**: DevTools middleware runs in production builds
- **Impact**: Medium - Adds overhead to store initialization
- **Status**: DevTools enabled in all environments
- **Recommendation**: Only enable devtools in development mode

#### 4. **Multiple Large Icon Imports** ⚠️ MEDIUM
- **Location**: `src/components/dashboard/NavigationSidebar.tsx`
- **Issue**: 12 icons imported from lucide-react
- **Impact**: Medium - Icon library can be large
- **Status**: Icons are tree-shakeable but many imported
- **Recommendation**: 
  - Verify tree-shaking is working
  - Consider importing icons individually if not already

#### 5. **Google Fonts Loading** ⚠️ LOW-MEDIUM
- **Location**: `src/app/layout.tsx`
- **Issue**: Inter font loads from Google Fonts
- **Impact**: Low-Medium - Network request, but uses `display: swap`
- **Status**: Optimized with `display: swap`
- **Recommendation**: Consider self-hosting fonts for better control

#### 6. **Radix UI Components** ⚠️ LOW
- **Location**: Multiple components
- **Issue**: Many Radix UI components imported
- **Impact**: Low - Components are well-optimized and tree-shakeable
- **Status**: Already optimized in `next.config.ts` with `optimizePackageImports`
- **Recommendation**: Monitor bundle size

#### 7. **Chart Components Lazy Loading** ✅ GOOD
- **Location**: `src/components/dashboard/Dashboard.tsx`
- **Status**: All 8 chart components are lazy-loaded with Suspense
- **Impact**: Positive - Prevents blocking initial render
- **Recommendation**: Keep as-is

### Optimization Recommendations (Priority Order)

#### Priority 1: Critical Fixes
1. **Defer GSAP Loading**
   - Only load GSAP when user clicks sidebar toggle
   - Use CSS transitions for initial render
   - Estimated improvement: 2-5 seconds

2. **Conditional DevTools**
   - Only enable Zustand devtools in development
   - Estimated improvement: 0.5-1 second

#### Priority 2: High Impact
3. **Optimize Recharts Loading**
   - Use dynamic imports for Recharts library itself
   - Tree-shake unused chart components
   - Estimated improvement: 1-3 seconds

4. **Font Optimization**
   - Self-host Inter font
   - Preload critical font weights
   - Estimated improvement: 0.5-1 second

#### Priority 3: Medium Impact
5. **Icon Optimization**
   - Verify lucide-react tree-shaking
   - Consider icon subset if needed
   - Estimated improvement: 0.2-0.5 seconds

6. **Bundle Analysis**
   - Run `npm run build` and analyze bundle sizes
   - Identify other large dependencies
   - Estimated improvement: Variable

### Next Steps

1. ✅ GSAP caching and CSS fallback (completed)
2. ⏳ Defer GSAP loading until user interaction
3. ⏳ Conditional Zustand devtools
4. ⏳ Recharts dynamic import optimization
5. ⏳ Font self-hosting
6. ⏳ Bundle size analysis

### Testing

After optimizations, test:
- Initial page load time
- Time to Interactive (TTI)
- First Contentful Paint (FCP)
- Largest Contentful Paint (LCP)
- Bundle size reduction

