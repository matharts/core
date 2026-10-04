# 全库兼容层清理

日期：2026-10-04。授权：用户明确选择“全库：清理旧接口、旧编码和兼容开关”。状态：本地实现与完整验证完成。

## 改动

- 删除空 `std` feature；默认无特性，唯一可选生产特性为 `serde`。hooks 与 CI 精简为 Serde 开／关 × debug／release。
- `CyclicRing::MODULUS` 改为 `NonZeroU8`，删除用于保留 `u8` 签名的私有 `Modulus<T>` 校验层。
- 删除 `Branch::three_combination_partners()`，完整三合组通过 `three_combination()` 查询。
- `SexagenaryCycle::nayin()` 返回 `Nayin`，删除 `nayin_identity()`；五行由 `Nayin::element()` 查询。
- 删除干支上的旬首、缺位二支和缺位判定旧入口；查询归属 `Xun`，补齐 `Xun::is_void_branch()`。
- `HiddenStems` 的编码字段使用 `tertiary`，拒绝 `residual`；三个字段都必须出现，空槽位使用 `null`。
- 所有可读枚举统一仅接受代码字符串，所有可读记录统一仅接受具名对象，并拒绝未知、重复和缺失字段。记录数组与 enum unit 对象不再接受。
- 枚举输出改为各领域类型上的 Serde 派生，移除为旧代码与数字索引准备的手写输出映射；严格输入和合法干支校验保留内联实现。
- 更新独立协议样本为 `encoding-v2.json`，同步所有仓内调用、测试、文档、hooks 与 CI。

完整调用替换与当前编码契约见[API 与编码变更](migration.md)。本次未添加旧名称别名、自动迁移或双版本解码。合法值、固定领域关系、输入方向、`no_std` 与 unsafe 禁用仍由当前验收检查。

## 验证

以下均为本次修改后的实际结果，无警告。矩阵只有默认无特性和 `--features serde` 两种配置，各执行 debug／release。

| 检查 | 结果 |
| --- | --- |
| `rtk proxy mise exec -- hk check --all --slow` | fmt、严格 Clippy、两组 debug 测试通过 |
| `cargo test --release --locked`，另加 `--features serde` | 两组 release 测试通过 |
| 测试计数 | Serde 关闭为 41 个集成测试，开启为 60 个；每组另有 16 个文档测试，含 1 个零周期 compile-fail |
| `thumbv7m-none-eabi` 实际库构建 | 无默认特性，以及另启用 Serde，两组通过 |
| `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --all-features --locked` | 严格文档构建通过 |
| `cargo run --example ganzhi` 的 dev／release × Serde 开／关 | 四次实际运行通过；旬、纳音、三合和三会输出均与预期匹配 |
| Feature 元数据与旧开关拒绝 | 元数据仅含 `default: []`、`serde`；请求 `--features std` 按预期失败，报 feature 不存在 |
| 生产源码检查 | 无旧查询入口、`residual` 编码、空 `std` feature 或私有周期兼容校验层残留 |
| 文档与工作区检查 | 18 份当前／历史文档的本地文件链接有效、无行尾空白；`git diff --check` 通过 |

独立测试继续覆盖全部合法干支及非法配对、极端周期位移、旬与纳音分割、三合／三会各 1728 个输入、全部爻卦结构及当前编码模型。新负例明确拒绝旧 JSON 形状和字段；可读与紧凑输入分别验收，重复 JSON 键直接用原始文本检查。

协议样本 schema 为 2，藏干字段为 `tertiary`，旧样本文件已移除。样本来自独立定义，未用生产编码器自动生成预期。

## 判断与边界

Jev 对清理方案选择给出 `clean`，置信度 0.99，未升级调查。依据是用户明确的全库范围和当前源码中的兼容逻辑；该判断不替代测试或授权。

最终 Jev 根据上述实际检查和旧入口清理证据判断本地工作完成，置信度 0.96，未升级调查。结论仅覆盖所提供的本地证据，不能证明下游升级、远端 CI 或具体第三方格式的字节兼容。

历史审查与实施文件保留为其时点记录，当前契约以此文件、迁移说明、源码与测试为准。本次只完成本地工作，不提交、推送或发布，不声称远端 CI 或下游工程已升级。
