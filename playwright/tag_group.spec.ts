import { test, expect } from "@playwright/test";

test("tag group selection and removal", async ({ page }) => {
  await page.goto("http://127.0.0.1:8080/component/?name=tag_group&");

  const bug = page.getByRole("row", { name: "bug" });
  const core = page.getByRole("row", { name: "core" });
  const feature = page.getByRole("row", { name: "feature" });

  await expect(bug).toHaveAttribute("data-selected", "true");
  await expect(core).toHaveAttribute("data-selected", "false");

  await core.click();
  await expect(core).toHaveAttribute("data-selected", "true");

  await feature.click();
  await expect(feature).toHaveAttribute("data-selected", "false");
  await expect(feature).toHaveAttribute("data-disabled", "true");

  await page.getByRole("button", { name: "Remove item bug" }).click();
  await expect(bug).toHaveCount(0);
});
