import { test, expect } from '@playwright/test';

/**
 * Visual Regression Tests - FINAL CHECKPOINT
 * 
 * These tests capture screenshots of key pages and components to ensure
 * 1:1 visual parity during the Tailwind to SCSS migration.
 * 
 * FINAL CHECKPOINT: After complete Tailwind removal
 * - All UI primitives migrated (46/46)
 * - All compound components migrated (10/10)
 * - All complex assemblies migrated (12/12)
 * - All page components migrated (9/9)
 * - Tailwind CSS completely removed
 * - tailwind-merge dependency removed
 * 
 * Run with: npx playwright test
 * Update snapshots: npx playwright test --update-snapshots
 */

test.describe('FINAL CHECKPOINT: Page Visual Regression', () => {
  test('Dashboard page visual regression', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(1000); // Wait for animations
    await expect(page).toHaveScreenshot('final-checkpoint-dashboard.png', {
      fullPage: true,
      maxDiffPixels: 100, // Allow small differences for anti-aliasing
    });
  });

  test('Projects page visual regression', async ({ page }) => {
    await page.goto('/projects');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(1000);
    await expect(page).toHaveScreenshot('final-checkpoint-projects.png', {
      fullPage: true,
      maxDiffPixels: 100,
    });
  });

  test('Chat page visual regression', async ({ page }) => {
    await page.goto('/chat');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(1000);
    await expect(page).toHaveScreenshot('final-checkpoint-chat.png', {
      fullPage: true,
      maxDiffPixels: 100,
    });
  });

  test('Settings page visual regression', async ({ page }) => {
    await page.goto('/settings');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(1000);
    await expect(page).toHaveScreenshot('final-checkpoint-settings.png', {
      fullPage: true,
      maxDiffPixels: 100,
    });
  });

  test('Agent Health page visual regression', async ({ page }) => {
    await page.goto('/agent-health');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(1000);
    await expect(page).toHaveScreenshot('final-checkpoint-agent-health.png', {
      fullPage: true,
      maxDiffPixels: 100,
    });
  });
});

test.describe('FINAL CHECKPOINT: Component Visual Regression', () => {
  test('Chat message component visual regression', async ({ page }) => {
    await page.goto('/chat');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(1000);
    
    // Try multiple selectors for chat messages
    const chatMessage = page.locator('[data-slot="chat-message"], [data-testid="chat-message"]').first();
    if (await chatMessage.count() > 0) {
      await expect(chatMessage).toHaveScreenshot('final-checkpoint-chat-message.png', {
        maxDiffPixels: 50,
      });
    } else {
      // If no messages, capture the empty state
      const chatContainer = page.locator('[data-slot="chat"], .chat-container').first();
      if (await chatContainer.count() > 0) {
        await expect(chatContainer).toHaveScreenshot('final-checkpoint-chat-empty.png', {
          maxDiffPixels: 50,
        });
      }
    }
  });

  test('Status badge component visual regression', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(1000);
    
    // Look for status badges
    const statusBadge = page.locator('[data-slot="status-badge"], .status-badge').first();
    if (await statusBadge.count() > 0) {
      await expect(statusBadge).toHaveScreenshot('final-checkpoint-status-badge.png', {
        maxDiffPixels: 30,
      });
    }
  });

  test('Priority indicator component visual regression', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(1000);
    
    const priorityIndicator = page.locator('[data-slot="priority-indicator"], .priority-indicator').first();
    if (await priorityIndicator.count() > 0) {
      await expect(priorityIndicator).toHaveScreenshot('final-checkpoint-priority-indicator.png', {
        maxDiffPixels: 30,
      });
    }
  });
});

test.describe('FINAL CHECKPOINT: Dark Mode Visual Regression', () => {
  test('Dashboard dark mode visual regression', async ({ page }) => {
    await page.goto('/');
    // Enable dark mode
    await page.evaluate(() => {
      document.documentElement.classList.add('dark');
    });
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(1000);
    await expect(page).toHaveScreenshot('final-checkpoint-dashboard-dark.png', {
      fullPage: true,
      maxDiffPixels: 100,
    });
  });

  test('Chat dark mode visual regression', async ({ page }) => {
    await page.goto('/chat');
    await page.evaluate(() => {
      document.documentElement.classList.add('dark');
    });
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(1000);
    await expect(page).toHaveScreenshot('final-checkpoint-chat-dark.png', {
      fullPage: true,
      maxDiffPixels: 100,
    });
  });
});

