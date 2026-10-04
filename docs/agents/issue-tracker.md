---
meta:
  contentType: "How-to"
  title: "用 GitHub Issue 管理任务"
  navLabel: "Issue 操作"
  category: "Agent 协作"
---

# 用 GitHub Issue 管理任务

在 `matharts/core` 的 GitHub Issues 中记录任务与规格。使用 `gh` 命令行工具，并按[项目约定](../../AGENTS.md)通过 RTK 执行命令；外部操作仍需相应授权。

## 操作约定

按操作选用下列命令。将 `42` 替换为目标 Issue 编号，将 `issue-body.md` 替换为已保存的 UTF-8 正文文件。

| 操作 | 命令 |
| --- | --- |
| 创建 | `rtk proxy gh issue create --repo matharts/core --title "完善规则来源" --body-file issue-body.md` |
| 读取 | `rtk proxy gh issue view 42 --repo matharts/core --comments` |
| 列表 | `rtk proxy gh issue list --repo matharts/core --state open --json number,title,body,labels` |
| 评论 | `rtk proxy gh issue comment 42 --repo matharts/core --body-file issue-body.md` |
| 添加标签 | `rtk proxy gh issue edit 42 --repo matharts/core --add-label needs-info` |
| 移除标签 | `rtk proxy gh issue edit 42 --repo matharts/core --remove-label needs-info` |
| 关闭 | `rtk proxy gh issue close 42 --repo matharts/core` |

每条命令独立执行，多行正文通过 `--body-file` 传入。技能要求“publish to the issue tracker”时创建 GitHub Issue；要求“fetch the relevant ticket”时读取对应 Issue 和评论。

## 区分 Issue 与 PR

**PRs as a request surface: no.**

本仓库通过 Issue 接收任务，不通过拉取请求（Pull Request，PR）接收任务。GitHub Issues 与 PR 共用编号；仅拿到编号时，先核实对象类型。

## 维护 Wayfinding 任务地图

Wayfinding 技能用带 `wayfinder:map` 标签的 Issue 记录任务地图。地图包含 `Notes`、`Decisions-so-far` 和 `Fog`；每个子任务单独建立 Issue。

按子任务类型选择标签：

- `wayfinder:research`
- `wayfinder:prototype`
- `wayfinder:grilling`
- `wayfinder:task`

优先使用 GitHub 原生子 Issue 和依赖关系。无法使用时，在地图中维护任务列表，并在子任务中注明归属与阻塞项，例如 `Part of #42`、`Blocked by: #43`。

按以下顺序推进任务：

1. 确认任务无人认领，且所有阻塞项已经关闭。
2. 按地图顺序选取任务，并分配给执行者。
3. 完成后记录答案，关闭任务，再把结论链接补入地图。

外部操作遵守[项目授权规则](../../AGENTS.md)，技能流程不能替代操作授权。
