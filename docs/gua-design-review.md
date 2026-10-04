# 爻卦方案审查

> 历史记录：本文反映兼容层清理之前的设计或检查状态，不代表当前接口、编码、特性矩阵或验证结果。当前契约见 [API 与编码变更](migration.md)，本轮结果见 [全库清理记录](breaking-cleanup.md)。

日期：2026-10-04。审查对象：[领域设计](gua-design.md)、[设计样本](fixtures/gua-design-v1.json)、[术语表](../CONTEXT.md)，并对照 [Core 总体约束](core-plan.md)。

当前状态（2026-10-04）：**R1、R2 已修订并通过生产 G1 验收**，见[实现记录](gua-implementation.md)。下面保留初次审查及设计修订时的历史证据；当时的“未实现”“待 G1”描述仅对应原阶段。

初次审查结论：发现 **2 项 P2 设计契约／验收缺口**。结构划分与位序方向可以保留，先补齐下述两项再按 G1 实施。初次审查只新增本报告；下面的行号、九条负例数量及文件哈希均对应修订前快照。这不是已经存在的生产 `Hexagram` 缺陷报告，因为该类型尚未实现。

## R1 · P2：只接受对象的验收遗漏了长度正确的数组

位置：`gua-design.md` 第127、151行；`gua-design-v1.json` 第209行起的负例。

第127行明确要求拒绝非对象，但当前九条负例只用空数组覆盖数组输入。两个字段都合法且长度正好为2的数组，是不同的路径。用当前依赖的常规 `derive(Deserialize)` 与 `deny_unknown_fields` 构建隔离的双字段结构体，实际得到：

```text
existing_negative_cases_rejected=9/9
input=["Qian","Kun"]; decoded=Ok(Hexagram { lower: Qian, upper: Kun })
input={"lower":"Qian","upper":"Kun"}; decoded=Ok(Hexagram { lower: Qian, upper: Kun })
```

因此，照现有负例实现验收时，一个违反“只接受对象”约定的反序列化器仍可全部通过。空数组只能证明字段数量不足会失败，不能证明数组形状被拒绝；`deny_unknown_fields` 也不能封闭这条路径。Serde 官方的[结构体反序列化示例](https://serde.rs/deserialize-struct.html)分别列有序列和映射访问路径。

修订建议：

- 加入 `['Qian', 'Kun']` 对应的合法 JSON 数组反例，即 `["Qian","Kun"]`，并同时测试 `from_str` 和 `from_value`。
- 若保持只接受 JSON 对象，明确反序列化 Visitor 不接受序列路径；字段表、缺失、重复、未知字段的检查继续保留。可采用手写 Visitor；不能只依赖派生加 `deny_unknown_fields`。
- 将限制范围写清楚：仅约束 JSON，还是也约束非人类可读 Serde 格式。后者有的格式会以位置序列表示结构体，应与 R2 一并确定，不能为了封堵 JSON 数组而暗改其他格式的约定。

验收：对象正例成功，长度为2且元素合法的数组失败；九条旧负例继续失败，原始 JSON 重复键负例也实际执行。

## R2 · P2：只固定 JSON 形状，尚未固定 Serde 数据模型

位置：`gua-design.md` 第119–129、151行；要求依据为 `core-plan.md` 第125行。

总体约束明确要求固定 Serde 数据模型。当前设计只规定代码字符串、两个对象字段与 JSON 编解码样本，没有确定四个新增类型对应的模型类别、稳定类型名及枚举 variant 索引等内容。

隔离复现中，派生枚举 `Trigram::Qian` 使用 unit variant，而手写 `serialize_str("Qian")` 使用 string；两者的 JSON 输出完全一致：

```text
unit_variant_json="Qian"; string_json="Qian"; equal=true
```

所以仅靠现有 JSON 验收无法发现 unit variant 改成 string 的变化，也无法区分 struct 改成 map 或结构体／枚举类型名发生变化。JSON 兼容与 Serde 模型兼容是不同的承诺。[Serde 官方数据模型](https://serde.rs/data-model.html)明确区分这些类别；[Serialize 实现文档](https://serde.rs/impl-serialize.html)说明 Rust 类型可以通过不同的 Serializer 入口映射到它们。

修订建议：

- 列出 `Trigram`、两种爻位和 `Hexagram` 的模型类别、稳定类型名、variant 代码与索引、字段名及字段序列。若采用 enum/struct，显式固定相应 Serde 名称。
- 增加独立于 JSON 的模型断言，例如 `serde_test` 的 token 测试或小型测试 serializer；它们只作为开发依赖／测试代码，不改变生产库的 `no_std` 目标。
- 如果本项目实际只打算承诺 JSON，应显式调整总体规划的兼容范围。不能把“不承诺第三方二进制字节兼容”解释成已经撤销了 Serde 数据模型承诺。

验收：把 unit variant 改成 string，或改动模型类型名，应触发对应模型测试失败，即使 JSON 输出仍相同。

## 其他审查结论

- 未发现八卦位表、泰否上下卦、阳阴显式转换、完整倒序与逐爻取反的逻辑错误。复用 `Primitive`、将老少动静留在上层、避免不具名的卦序回绕，均有明确边界。
- 延后六十四卦名称与文王卦序是方案写明的阶段范围，不作为缺陷；所选文献样本也没有被写成已经完成全部流派校核。
- 建议保存 G0 算术校验的可重跑脚本和命令；当前仓库保存了样本与统计记录，但没有对应 runner。这是证据可复现性的改进建议，不列为阻断项。

## 实际复现与限制

实测环境：Rust 1.99.0、Serde 1.0.229、serde_json 1.0.151。复现程序位于 `/tmp/matharts-gua-design-review-20261004.rs`，通过相同 `serde_core` 构建指纹的本地依赖缓存编译后执行；退出码0，断言九条已有负例全部失败，并观测到长度正确的数组被接受。

核心复现如下，可在使用上述版本及 Serde derive 特性的隔离程序中重跑：

```rust
use serde::{Deserialize, Serialize, Serializer};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "Trigram")]
enum Trigram { Qian, Kun }

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "Hexagram", deny_unknown_fields)]
struct Hexagram { lower: Trigram, upper: Trigram }

struct StringCode;
impl Serialize for StringCode {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str("Qian")
    }
}

fn main() {
    assert!(serde_json::from_str::<Hexagram>(r#"["Qian","Kun"]"#).is_ok());
    assert_eq!(
        serde_json::to_string(&Trigram::Qian).unwrap(),
        serde_json::to_string(&StringCode).unwrap(),
    );
}
```

这段代码刻意演示常规派生不能满足方案约束，不是对不存在的生产实现作推测性漏洞认定。未运行无关的整个 Rust 功能矩阵；本轮新增的仅为审查报告。

初次审查前后确认以下三个输入文件 SHA-256 不变（修订前历史快照，不是当前方案哈希）：

| 文件 | SHA-256 |
| --- | --- |
| `docs/gua-design.md` | `9a896d0386c784d0c31de9683740d2a1278f487fdc4b802e2dba1617ae900840` |
| `docs/fixtures/gua-design-v1.json` | `59a1b51773fe4a75fb3033959cd55f0ff9ad836e7bb1b3ce8994e6905b6077ac` |
| `CONTEXT.md` | `ccaab7681f5bdb578ceeb5b1597280b31f629babe0365c893ba51bfba04daa9c` |

Jev 依据上述实测与总体约束，支持将两项归为 P2，并建议保留结构方向、先修订序列化契约与验收。该判断仅辅助归类，不替代本机复现、未来实现验收或跨体系文献核验。

## 修订验证

- **R1**：显式区分人类可读与非人类可读输入；前者只接受 map，后者保留严格双元素结构体序列。设计样本新增长度正确数组、其他非对象值及原始重复键。16个非法值均实际执行 `from_str` 和 `from_value`；2个重复键字符串单独执行。常规派生接受双元素数组的对照仍可复现。
- **R2**：固定四个类型的模型类别、Serde 名称、17个 variant 的代码与索引、结构体长度和字段序列。输出模型不因输入模式变化。枚举文本输入严格走字符串入口，避免接受对象式 unit variant。模型检查识别6类变化，包括 JSON 相同的 string 替代、枚举重命名及索引变化。
- **可重跑证据**：[隔离探针](checks/gua-serde/README.md) 的四组测试、fmt 和严格 Clippy 通过；依赖仅属于隔离实验。`serde_test` 不观察输出 variant index，探针另有专用 serializer 检查该值。
- **范围限制**：以上只关闭方案与验收设计缺口；不是生产实现修复，也不是具体二进制格式兼容证明。G1 仍需独立64组合样本、生产接口测试与完整工程矩阵。G0 算术 runner 仍是原报告的非阻断建议，本次探针仅覆盖 Serde。

Jev 根据上述实测证据判定“方案层可以交付，G1 生产验收仍待执行”（置信度0.98）；它未将隔离探针结果判为生产实现完成。
