# Issue tracker: GitHub

Issues 和 PRD 以 GitHub issue 形式存在。所有操作使用 `gh` CLI。

## 规范

- **创建 issue**：`gh issue create --title "..." --body "..."`。多行正文使用 heredoc。
- **读取 issue**：`gh issue view <number> --comments`，用 `jq` 过滤评论并获取标签。
- **列出 issue**：`gh issue list --state open --json number,title,body,labels,comments --jq '[.[] | {number, title, body, labels: [.labels[].name], comments: [.comments[].body]}]'`，配合 `--label` 和 `--state` 过滤。
- **评论**：`gh issue comment <number> --body "..."`
- **应用 / 移除标签**：`gh issue edit <number> --add-label "..."` / `--remove-label "..."`
- **关闭**：`gh issue close <number> --comment "..."`

从 `git remote -v` 推断仓库 — `gh` 在克隆目录内自动处理。

## 当技能要求"发布到 issue tracker"

创建一个 GitHub issue。

## 当技能要求"获取相关 ticket"

运行 `gh issue view <number> --comments`。
