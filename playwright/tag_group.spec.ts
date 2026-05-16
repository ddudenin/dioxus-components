import { test, expect, type Page } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";

const BASE = process.env.PLAYWRIGHT_BASE_URL ?? "http://127.0.0.1:8080";
const URL = `${BASE}/component/?name=tag_group&`;
const LOAD_TIMEOUT = 20 * 60 * 1000;

function tag(page: Page, name: string) {
  return page.getByRole("row", { name });
}

async function loadTagGroup(page: Page) {
  await page.goto(URL, { timeout: LOAD_TIMEOUT });
  await expect(page.getByText("Labels", { exact: true })).toBeVisible({
    timeout: 30000,
  });
  await expect(page.getByRole("grid")).toBeVisible();
}

test.describe("Tag group", () => {
  test.beforeEach(async ({ page }) => {
    await loadTagGroup(page);
  });

  test.describe("Selection", () => {
    test("shows initial selection and supports multiple selection", async ({
      page,
    }) => {
      const bug = tag(page, "bug");
      const core = tag(page, "core");
      const desktop = tag(page, "desktop");

      await expect(bug).toHaveAttribute("data-selected", "true");
      await expect(core).toHaveAttribute("data-selected", "false");

      await core.click();
      await expect(bug).toHaveAttribute("data-selected", "true");
      await expect(core).toHaveAttribute("data-selected", "true");

      await desktop.click();
      await expect(desktop).toHaveAttribute("data-selected", "true");
    });

    test("does not clear the last selected tag when empty selection is disallowed", async ({
      page,
    }) => {
      const bug = tag(page, "bug");

      await expect(bug).toHaveAttribute("data-selected", "true");
      await bug.click();
      await expect(bug).toHaveAttribute("data-selected", "true");

      await tag(page, "core").click();
      await bug.click();
      await expect(bug).toHaveAttribute("data-selected", "false");
      await expect(tag(page, "core")).toHaveAttribute("data-selected", "true");
    });

    test("ignores clicks on disabled tags", async ({ page }) => {
      const feature = tag(page, "feature");
      const example = tag(page, "example");

      await expect(feature).toHaveAttribute("data-disabled", "true");
      await expect(example).toHaveAttribute("data-disabled", "true");

      await feature.click();
      await expect(feature).toHaveAttribute("data-selected", "false");

      await example.click();
      await expect(example).toHaveAttribute("data-selected", "false");
    });

    test("clears selection on Escape", async ({ page }) => {
      const bug = tag(page, "bug");
      await bug.click();
      await expect(bug).toBeFocused();

      await page.keyboard.press("Escape");
      await expect(bug).toHaveAttribute("data-selected", "false");
      await expect(tag(page, "core")).toHaveAttribute("data-selected", "false");
    });
  });

  test.describe("Keyboard", () => {
    test("roving focus skips disabled tags", async ({ page }) => {
      const bug = tag(page, "bug");
      const core = tag(page, "core");

      await bug.click();
      await expect(bug).toBeFocused();

      await page.keyboard.press("ArrowRight");
      await expect(core).toBeFocused();

      await page.keyboard.press("ArrowLeft");
      await expect(bug).toBeFocused();
    });

    test("Space toggles selection on the focused tag", async ({ page }) => {
      const core = tag(page, "core");

      await core.click();
      await expect(core).toBeFocused();
      await expect(core).toHaveAttribute("data-selected", "true");

      await page.keyboard.press("Space");
      await expect(core).toHaveAttribute("data-selected", "false");

      await page.keyboard.press("Space");
      await expect(core).toHaveAttribute("data-selected", "true");
    });

    test("Delete removes all selected tags", async ({ page }) => {
      const bug = tag(page, "bug");
      const core = tag(page, "core");

      await core.click();
      await expect(bug).toHaveAttribute("data-selected", "true");
      await expect(core).toHaveAttribute("data-selected", "true");

      await core.click();
      await page.keyboard.press("Delete");

      await expect(bug).toHaveCount(0);
      await expect(core).toHaveCount(0);
    });
  });

  test.describe("Removal", () => {
    test("remove button deletes a tag", async ({ page }) => {
      const bug = tag(page, "bug");
      await expect(bug).toBeVisible();

      await page.getByRole("button", { name: "Remove item bug" }).click();
      await expect(bug).toHaveCount(0);
    });
  });

  test.describe("Accessibility", () => {
    test("has no automatically detectable a11y violations", async ({ page }) => {
      const results = await new AxeBuilder({ page }).analyze();
      expect(results.violations).toEqual([]);
    });
  });
});
