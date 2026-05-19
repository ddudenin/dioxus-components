import { test, expect, type Page } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";

const BASE = process.env.PLAYWRIGHT_BASE_URL ?? "http://127.0.0.1:8080";
const URL = `${BASE}/component/?name=tag_group&`;
const LOAD_TIMEOUT = 20 * 60 * 1000;

function multiVariant(page: Page) {
  return page
    .locator(".dx-component-variant")
    .filter({ has: page.getByRole("heading", { name: "multi" }) });
}

function tag(page: Page, name: string) {
  return multiVariant(page).getByRole("row", { name });
}

async function loadTagGroup(page: Page) {
  await page.goto(URL, { timeout: LOAD_TIMEOUT });
  await expect(
    multiVariant(page).getByText("Labels", { exact: true }),
  ).toBeVisible({
    timeout: 30000,
  });
  await expect(multiVariant(page).getByRole("grid")).toBeVisible();
}

test.describe("Tag group", () => {
  // One page load at a time — parallel navigations contend with the preview webServer build.
  test.describe.configure({ mode: "serial" });

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

    test("marks disabled tags as non-interactive", async ({ page }) => {
      const feature = tag(page, "feature");
      const example = tag(page, "example");

      await expect(feature).toHaveAttribute("data-disabled", "true");
      await expect(feature).toHaveAttribute("aria-disabled", "true");
      await expect(feature).toHaveAttribute("data-selected", "false");

      await expect(example).toHaveAttribute("data-disabled", "true");
      await expect(example).toHaveAttribute("aria-disabled", "true");
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
      await expect(core).toBeFocused();

      await page.keyboard.press("Delete");

      await expect(bug).toHaveCount(0);
      await expect(core).toHaveCount(0);
    });
  });

  test.describe("Removal", () => {
    test("remove button deletes a tag", async ({ page }) => {
      const bug = tag(page, "bug");
      await expect(bug).toBeVisible();

      await multiVariant(page)
        .getByRole("button", { name: "Remove item bug" })
        .click();
      await expect(bug).toHaveCount(0);
    });
  });

  test.describe("Accessibility", () => {
    test("has no automatically detectable a11y violations on the tag list", async ({
      page,
    }) => {
      const results = await new AxeBuilder({ page })
        .include(".dx-component-variant [role=\"grid\"]")
        .disableRules(["color-contrast"])
        .analyze();
      expect(results.violations).toEqual([]);
    });
  });
});
