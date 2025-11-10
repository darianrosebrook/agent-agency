# Build Comparison Report

Generated: 11/9/2025, 10:24:43 PM

## Summary

- **Old Build CSS Rules**: 386
- **New Build CSS Rules**: 1788
- **Matching Rules**: 0
- **Different Rules**: 4
- **Missing in New**: 382
- **Missing in Old**: 1784

## Differences Found

### Rules Missing in New Version (382)

These CSS rules exist in the old Tailwind build but not in the new SCSS build:

- **pointer-events-none**: `pointer-events:none...`
- **sr-only**: `clip:rect(0,0,0,0);white-space:nowrap;border-width:0;width:1px;height:1px;margin:-1px;padding:0;posi...`
- **absolute**: `position:absolute...`
- **fixed**: `position:fixed...`
- **relative**: `position:relative...`
- **sticky**: `position:sticky...`
- **inset-0**: `inset:calc(var(--spacing) * 0)...`
- **-top-2**: `top:calc(var(--spacing) * -2)...`
- **top-0**: `top:calc(var(--spacing) * 0)...`
- **top-2**: `top:calc(var(--spacing) * 2)...`
- **top-4**: `top:calc(var(--spacing) * 4)...`
- **top-8**: `top:calc(var(--spacing) * 8)...`
- **top-full**: `top:100%...`
- **-right-2**: `right:calc(var(--spacing) * -2)...`
- **right-0**: `right:calc(var(--spacing) * 0)...`
- **right-2**: `right:calc(var(--spacing) * 2)...`
- **right-3**: `right:calc(var(--spacing) * 3)...`
- **right-4**: `right:calc(var(--spacing) * 4)...`
- **right-8**: `right:calc(var(--spacing) * 8)...`
- **-bottom-2**: `bottom:calc(var(--spacing) * -2)...`
- **-bottom-3**: `bottom:calc(var(--spacing) * -3)...`
- **bottom-8**: `bottom:calc(var(--spacing) * 8)...`
- **-left-3**: `left:calc(var(--spacing) * -3)...`
- **left-0**: `left:calc(var(--spacing) * 0)...`
- **left-2**: `left:calc(var(--spacing) * 2)...`
- **left-3**: `left:calc(var(--spacing) * 3)...`
- **z-10**: `z-index:10...`
- **z-20**: `z-index:20...`
- **z-50**: `z-index:50...`
- **col-span-1**: `grid-column:span 1 / span 1...`
- **col-span-4**: `grid-column:span 4 / span 4...`
- **col-span-5**: `grid-column:span 5 / span 5...`
- **col-span-7**: `grid-column:span 7 / span 7...`
- **col-span-8**: `grid-column:span 8 / span 8...`
- **col-span-12**: `grid-column:span 12 / span 12...`
- **row-span-2**: `grid-row:span 2 / span 2...`
- **row-span-3**: `grid-row:span 3 / span 3...`
- **row-span-6**: `grid-row:span 6 / span 6...`
- **container**: `max-width:96rem...`
- **m-4**: `margin:calc(var(--spacing) * 4)...`
- **-mx-1**: `margin-inline:calc(var(--spacing) * -1)...`
- **-mx-2**: `margin-inline:calc(var(--spacing) * -2)...`
- **mx-auto**: `margin-inline:auto...`
- **my-1**: `margin-block:calc(var(--spacing) * 1)...`
- **-mt-2**: `margin-top:calc(var(--spacing) * -2)...`
- **mt-0**: `margin-top:calc(var(--spacing) * 0)...`
- **mt-1**: `margin-top:calc(var(--spacing) * 1)...`
- **5**: `gap:calc(var(--spacing) * 2.5)...`
- **mt-2**: `margin-top:calc(var(--spacing) * 2)...`
- **mt-3**: `margin-top:calc(var(--spacing) * 3)...`


*... and 332 more*

### Rules Only in New Version (1784)

These CSS rules exist in the new SCSS build but not in the old Tailwind build:

- **SecuritySettingsTab_animate-spin-slow__mNSuH**: `animation:SecuritySettingsTab_spin-slow__dlB0I 3s linear infinite...`
- **SecuritySettingsTab_animate-spin__Ale7A**: `animation:SecuritySettingsTab_spin__5L_ji 1s linear infinite...`
- **SecuritySettingsTab_animate-pulse__XyYDy**: `animation:SecuritySettingsTab_pulse__jaCmW 2s cubic-bezier(.4,0,.6,1) infinite...`
- **SecuritySettingsTab_animate-fade-in__oABY2**: `animation:SecuritySettingsTab_fade-in__ug5Td .25s cubic-bezier(0,0,.2,1)...`
- **SecuritySettingsTab_animate-fade-out__AnPMK**: `animation:SecuritySettingsTab_fade-out__mIjMo .25s cubic-bezier(0,0,.2,1)...`
- **SecuritySettingsTab_animate-slide-in__1qeCw**: `animation:SecuritySettingsTab_slide-in__lZX7P .25s cubic-bezier(0,0,.2,1)...`
- **SecuritySettingsTab_animate-slide-out__SYtoD**: `animation:SecuritySettingsTab_slide-out__iDSyR .25s cubic-bezier(0,0,.2,1)...`
- **SecuritySettingsTab_securityTab__EJ6p_**: `display:flex;flex-direction:column;gap:1.5rem...`
- **SecuritySettingsTab_loading__wMevy**: `padding:2rem;text-align:center;color:#a1a1aa...`
- **SecuritySettingsTab_form__E72ZU**: `display:flex;flex-direction:column;gap:1rem...`
- **SecuritySettingsTab_formGroup__ZVYBv**: `display:flex;flex-direction:column;gap:.5rem...`
- **SecuritySettingsTab_twoFAEnabled__4AptM**: `display:flex;flex-direction:column;gap:1rem...`
- **SecuritySettingsTab_qrCode__HakOP**: `display:flex;justify-content:center;padding:1rem;background-color:#fff;border-radius:.75rem...`
- **SecuritySettingsTab_backupCodes__D_us8**: `margin-top:1rem;padding:1rem;background-color:#0f0f0f;border-radius:.75rem...`
- **ApiKeysTab_animate-spin-slow__NfAd0**: `animation:ApiKeysTab_spin-slow__LzgyP 3s linear infinite...`
- **ApiKeysTab_animate-spin__CGXZt**: `animation:ApiKeysTab_spin__dX7_d 1s linear infinite...`
- **ApiKeysTab_animate-pulse__2Aybd**: `animation:ApiKeysTab_pulse__scg0H 2s cubic-bezier(.4,0,.6,1) infinite...`
- **ApiKeysTab_animate-fade-in__kCEpo**: `animation:ApiKeysTab_fade-in__7_tyc .25s cubic-bezier(0,0,.2,1)...`
- **ApiKeysTab_animate-fade-out__8qcMA**: `animation:ApiKeysTab_fade-out__z82Qj .25s cubic-bezier(0,0,.2,1)...`
- **ApiKeysTab_animate-slide-in__jo_Kw**: `animation:ApiKeysTab_slide-in__03iCl .25s cubic-bezier(0,0,.2,1)...`
- **ApiKeysTab_animate-slide-out__3PS12**: `animation:ApiKeysTab_slide-out__p92s3 .25s cubic-bezier(0,0,.2,1)...`
- **ApiKeysTab_apiKeysTab__b_lV7**: `display:flex;flex-direction:column;gap:1.5rem...`
- **ApiKeysTab_loading__GDZ8U**: `padding:2rem;text-align:center;color:#a1a1aa...`
- **ApiKeysTab_newKeyAlert__b5Fjj**: `padding:1rem;background-color:#0f0f0f;border:1px solid #a1a1aa;border-radius:calc(var(--radius) - 2p...`
- **ApiKeysTab_createForm__jp5g8**: `margin-top:1rem;padding:1rem;background-color:#0f0f0f;border-radius:calc(var(--radius) - 2px);displa...`
- **ApiKeysTab_formGroup__QDieL**: `display:flex;flex-direction:column;gap:.5rem...`
- **ApiKeysTab_keysList__NYrLz**: `margin-top:1.5rem;display:flex;flex-direction:column;gap:1rem...`
- **ApiKeysTab_keyItem__bUXRm**: `display:flex;justify-content:space-between;align-items:center;padding:1rem;background-color:#0f0f0f;...`
- **ApiKeysTab_keyActions__gDAoV**: `display:flex;gap:.5rem...`
- **IntegrationsTab_animate-spin-slow__b5oYV**: `animation:IntegrationsTab_spin-slow__HC5i5 3s linear infinite...`
- **IntegrationsTab_animate-spin__vPPZQ**: `animation:IntegrationsTab_spin__NpBLk 1s linear infinite...`
- **IntegrationsTab_animate-pulse__V5jbG**: `animation:IntegrationsTab_pulse__F_awe 2s cubic-bezier(.4,0,.6,1) infinite...`
- **IntegrationsTab_animate-fade-in__pLrgO**: `animation:IntegrationsTab_fade-in__btRJs .25s cubic-bezier(0,0,.2,1)...`
- **IntegrationsTab_animate-fade-out__euyKO**: `animation:IntegrationsTab_fade-out__4AKnB .25s cubic-bezier(0,0,.2,1)...`
- **IntegrationsTab_animate-slide-in__Solob**: `animation:IntegrationsTab_slide-in__eRi3u .25s cubic-bezier(0,0,.2,1)...`
- **IntegrationsTab_animate-slide-out__kePQu**: `animation:IntegrationsTab_slide-out__HVLOe .25s cubic-bezier(0,0,.2,1)...`
- **IntegrationsTab_integrationsTab__Of3kP**: `display:flex;flex-direction:column;gap:1.5rem...`
- **IntegrationsTab_loading__sjbWb**: `padding:2rem;text-align:center;color:#a1a1aa...`
- **IntegrationsTab_integrationsList__1HyHC**: `display:flex;flex-direction:column;gap:1rem...`
- **IntegrationsTab_integrationItem__mklnr**: `display:flex;justify-content:space-between;align-items:center;padding:1rem;background-color:#0f0f0f;...`
- **IntegrationsTab_integrationActions__ybpNp**: `display:flex;gap:.5rem...`
- **page_animate-spin-slow__pn5_y**: `animation:page_spin-slow__hOZDI 3s linear infinite...`
- **page_animate-spin__puYBy**: `animation:page_spin__facpq 1s linear infinite...`
- **page_animate-pulse__l7dy1**: `animation:page_pulse__Y50G9 2s cubic-bezier(.4,0,.6,1) infinite...`
- **page_animate-fade-in__oy2ma**: `animation:page_fade-in__VxZ6N .25s cubic-bezier(0,0,.2,1)...`
- **page_animate-fade-out__PKdMf**: `animation:page_fade-out__EsldE .25s cubic-bezier(0,0,.2,1)...`
- **page_animate-slide-in__B9za0**: `animation:page_slide-in__GLj1Z .25s cubic-bezier(0,0,.2,1)...`
- **page_animate-slide-out__COvkl**: `animation:page_slide-out__P0TpV .25s cubic-bezier(0,0,.2,1)...`
- **page_settingsPage__NPQXd**: `width:100%;min-height:100%;padding:2rem;max-width:80rem;margin-left:auto;margin-right:auto...`
- **page_settingsHeader__vDHXe**: `margin-bottom:2rem...`


*... and 1734 more*

### Rules with Different Properties (4)

These CSS rules exist in both but have different properties:


#### animate-pulse

**Old:**
```css
animation:var(--animate-pulse)
```

**New:**
```css
animation:pulse 2s cubic-bezier(.4,0,.6,1) infinite
```


#### animate-spin

**Old:**
```css
animation:var(--animate-spin)
```

**New:**
```css
animation:spin 1s linear infinite
```


#### dark

**Old:**
```css
--background: oklch(.145 0 0);--foreground: oklch(.95 0 0);--card: oklch(.145 0 0);--card-foreground: oklch(.95 0 0);--popover: oklch(.145 0 0);--popover-foreground: oklch(.95 0 0);--primary: oklch(.985 0 0);--primary-foreground: oklch(.205 0 0);--secondary: oklch(.269 0 0);--secondary-foreground: oklch(.95 0 0);--muted: oklch(.269 0 0);--muted-foreground: oklch(.75 0 0);--accent: oklch(.269 0 0);--accent-foreground: oklch(.95 0 0);--destructive: oklch(.396 .141 25.723);--destructive-foreground: oklch(.637 .237 25.331);--border: oklch(.269 0 0);--input: oklch(.269 0 0);--ring: oklch(.439 0 0);--font-weight-semibold: 600;--font-weight-medium: 500;--font-weight-normal: 400;--chart-1: oklch(.488 .243 264.376);--chart-2: oklch(.696 .17 162.48);--chart-3: oklch(.769 .188 70.08);--chart-4: oklch(.627 .265 303.9);--chart-5: oklch(.645 .246 16.439);--sidebar: oklch(.205 0 0);--sidebar-foreground: oklch(.95 0 0);--sidebar-primary: oklch(.488 .243 264.376);--sidebar-primary-foreground: oklch(.95 0 0);--sidebar-accent: oklch(.269 0 0);--sidebar-accent-foreground: oklch(.95 0 0);--sidebar-border: oklch(.269 0 0);--sidebar-ring: oklch(.439 0 0)
```

**New:**
```css
--background:oklch(0.145 0 0);--foreground:oklch(0.95 0 0);--card:oklch(0.145 0 0);--card-foreground:oklch(0.95 0 0);--popover:oklch(0.145 0 0);--popover-foreground:oklch(0.95 0 0);--primary:oklch(0.985 0 0);--primary-foreground:oklch(0.205 0 0);--secondary:oklch(0.269 0 0);--secondary-foreground:oklch(0.95 0 0);--muted:oklch(0.269 0 0);--muted-foreground:oklch(0.75 0 0);--accent:oklch(0.269 0 0);--accent-foreground:oklch(0.95 0 0);--destructive:oklch(0.396 0.141 25.723);--destructive-foreground:oklch(0.637 0.237 25.331);--border:oklch(0.269 0 0);--input:oklch(0.269 0 0);--ring:oklch(0.439 0 0);--font-weight-semibold:600;--font-weight-medium:500;--font-weight-normal:400;--chart-1:oklch(0.488 0.243 264.376);--chart-2:oklch(0.696 0.17 162.48);--chart-3:oklch(0.769 0.188 70.08);--chart-4:oklch(0.627 0.265 303.9);--chart-5:oklch(0.645 0.246 16.439);--sidebar:oklch(0.205 0 0);--sidebar-foreground:oklch(0.95 0 0);--sidebar-primary:oklch(0.488 0.243 264.376);--sidebar-primary-foreground:oklch(0.95 0 0);--sidebar-accent:oklch(0.269 0 0);--sidebar-accent-foreground:oklch(0.95 0 0);--sidebar-border:oklch(0.269 0 0);--sidebar-ring:oklch(0.439 0 0)
```


#### animate-spin-slow

**Old:**
```css
animation:3s linear infinite spin-slow
```

**New:**
```css
animation:spin-slow 3s linear infinite
```




## Next Steps

1. Review the differences above
2. Verify which missing rules are intentional (e.g., unused Tailwind utilities)
3. Fix any unintentional differences
4. Re-run this script to verify fixes

## Full Report

See `build-comparison-report.json` for the complete data.
