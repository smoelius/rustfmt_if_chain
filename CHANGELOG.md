# Changelog

## 0.1.5

- Handle `if_chain` invocations in expression positions (e.g., `let x = if_chain! { ... };`) ([#27](https://github.com/smoelius/rustfmt_if_chain/pull/27))

## 0.1.4

- An incorrect internal check was causing `rustfmt_if_chain` to panic when applied to multiple files. The check is now corrected and guarded by a feature. ([0fa6a80](https://github.com/smoelius/rustfmt_if_chain/commit/0fa6a80328de648729261496bc930270acbc2b48))

## 0.1.3

- Update README.md ([42dd5f4](https://github.com/smoelius/rustfmt_if_chain/commit/42dd5f436876755b40f2532de585807bf411aa51))
- If no paths are passed, run `rustfmt` anyway ([7000c20](https://github.com/smoelius/rustfmt_if_chain/commit/7000c204de9148f1dd0af9b4861d24f0f312c4af))

## 0.1.2

- Correct error message ([faf6c75](https://github.com/smoelius/rustfmt_if_chain/commit/faf6c75d1615db26b2ba5a18a7a979f6409e77fd))
- Wrap usage to 72 columns ([dad3138](https://github.com/smoelius/rustfmt_if_chain/commit/dad3138342cdf675a71f8137c028dcc1c430e58c))
