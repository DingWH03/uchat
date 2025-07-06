# Contributing Guide

欢迎你为本项目贡献代码、文档或建议！

## 如何贡献

1. **Fork 本仓库**
2. 新建分支：`git checkout -b my-feature`
3. 提交更改：`git commit -am 'Add some feature'`
4. 推送分支：`git push origin my-feature`
5. 在 GitHub 上创建 Pull Request

## 代码规范

- 保持代码整洁、风格统一。请在提交前运行：

```bash
  cargo fmt
  cargo clippy
```

- 提交前确保所有测试通过：

  ```bash
  cargo test
  ```

## Issue 反馈

- Bug 报告、功能建议请在 [Issue](https://github.com/DingWH03/uchat/issues) 区提交
- 报告 Bug 时请附上复现步骤、期望行为、实际行为及相关日志（如有）

## Pull Request 说明

- 请描述你的更改内容与动机
- 关联相关 Issue（如适用）
- 保证 PR 不引入未使用依赖或无关更改

## 提交信息

- 推荐使用英文，简洁明了
- 格式建议：`[类型] 描述（#issue号）`

  - 示例: `fix: 修复登录接口超时 (#12)`

## 联系方式

如有疑问或合作意向，可通过 Issues 或邮件联系我。

感谢你的贡献！
