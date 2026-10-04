//! 从中文输入到领域查询的完整示例。
//!
//! 运行：`cargo run --example ganzhi --no-default-features --locked`。
//! 示例使用 `std` 打印结果；matharts-core 库本身仍为 `no_std`。

use matharts_core::{
    Branch, CyclicRing, Element, FiveCombination, InvalidGanzhi, Nayin, ParseError, Primitive,
    SexagenaryCycle, SixBreak, SixClash, SixCombination, SixHarm, Stem, ThreeCombination,
    ThreeMeeting, Xun,
};

fn main() -> Result<(), ParseError> {
    let ganzhi: SexagenaryCycle = "甲子".parse()?;
    let stem = ganzhi.stem();
    let branch = ganzhi.branch();
    assert_eq!((stem, branch), (Stem::Jia, Branch::Zi));
    assert_eq!(
        (stem.primitive(), stem.element()),
        (Primitive::Yang, Element::Wood)
    );
    assert_eq!(
        (branch.primitive(), branch.element()),
        (Primitive::Yang, Element::Water)
    );
    assert_eq!(ganzhi.to_string(), "甲子");
    println!("输入：{ganzhi}");
    println!(
        "天干：{stem}，{}{}",
        primitive_name(stem.primitive()),
        element_name(stem.element())
    );
    println!(
        "地支：{branch}，{}{}",
        primitive_name(branch.primitive()),
        element_name(branch.element())
    );
    let five = stem.five_combination();
    let six = branch.six_combination();
    assert_eq!(five, FiveCombination::JiaJi);
    assert_eq!(six, SixCombination::ZiChou);
    assert_eq!(five.members(), [Stem::Jia, Stem::Ji]);
    assert_eq!(six.members(), [Branch::Zi, Branch::Chou]);
    assert_eq!(five.element(), Element::Earth);
    assert_eq!(five.partner_of(stem), Some(Stem::Ji));
    assert_eq!(six.partner_of(branch), Some(Branch::Chou));
    assert_eq!(five.partner_of(Stem::Yi), None);
    assert_eq!(FiveCombination::from_stems([Stem::Ji, stem]), Some(five));
    assert_eq!(FiveCombination::from_stems([stem; 2]), None);
    assert_eq!(
        SixCombination::from_branches([Branch::Chou, branch]),
        Some(six)
    );
    assert_eq!(SixCombination::from_branches([branch, Branch::Yin]), None);
    let [a, b] = five.members();
    println!(
        "所属五合组：{a}、{b}；固定对应：{}",
        element_name(five.element())
    );
    let [a, b] = six.members();
    println!("所属六合组：{a}、{b}");
    show_branch_pairs(branch);
    show_branch_groups(branch);

    // 循环步进通过公共 CyclicRing trait 提供，与历法日期无关。
    let next = ganzhi.offset(1);
    let last: SexagenaryCycle = "癸亥".parse()?;
    assert_eq!((next.stem(), next.branch()), (Stem::Yi, Branch::Chou));
    assert_eq!(last.offset(1), ganzhi);
    println!(
        "顺进一位：{ganzhi} → {next}；周期回绕：{last} → {}",
        last.offset(1)
    );

    let xun = ganzhi.xun();
    let (first_void, second_void) = xun.void_branches();
    assert_eq!(xun, Xun::JiaZi);
    assert_eq!(xun.name(), "甲子旬");
    assert_eq!((first_void, second_void), (Branch::Xu, Branch::Hai));
    // 这里只查询旬内未出现的两支，不推断命盘中的空亡效应。
    println!(
        "所属旬：{}；缺位地支：{first_void}、{second_void}",
        xun.name()
    );

    let nayin = ganzhi.nayin();
    assert_eq!(nayin, Nayin::HaiZhongJin);
    assert_eq!(nayin.name(), "海中金");
    assert_eq!(ganzhi.nayin().element(), Element::Metal);
    println!(
        "纳音：{}；纳音五行：{}",
        nayin.name(),
        element_name(ganzhi.nayin().element())
    );

    // 严格输入不会自动 trim、接受拼音别名或修正阴阳不匹配的组合。
    // 固定预期断言在 debug 和 release 下均执行，行为变化会使示例失败。
    for (input, expected) in [
        (" 甲子", ParseError::InvalidGanzhiFormat),
        ("JiaZi", ParseError::InvalidGanzhiFormat),
        ("甲子年", ParseError::InvalidGanzhiFormat),
        ("A子", ParseError::InvalidStem),
        ("甲A", ParseError::InvalidBranch),
        ("甲丑", ParseError::InvalidGanzhi(InvalidGanzhi)),
    ] {
        let error = input
            .parse::<SexagenaryCycle>()
            .expect_err("应拒绝非法输入");
        assert_eq!(error, expected);
        let reason = match error {
            ParseError::InvalidGanzhiFormat => "必须恰好为两个中文干支字符",
            ParseError::InvalidStem => "天干名称无效",
            ParseError::InvalidBranch => "地支名称无效",
            ParseError::InvalidGanzhi(_) => "天干与地支阴阳不匹配",
            // ParseError 是 non_exhaustive，外部调用方应保留未知错误分支。
            _ => "未知解析错误",
        };
        println!("拒绝输入 {input:?}：{reason}");
    }

    Ok(())
}

fn show_branch_pairs(branch: Branch) {
    let clash = branch.six_clash();
    let harm = branch.six_harm();
    let breaking = branch.six_break();
    assert_eq!(clash, SixClash::ZiWu);
    assert_eq!(harm, SixHarm::ZiWei);
    assert_eq!(breaking, SixBreak::ZiYou);
    assert_eq!(clash.partner_of(branch), Some(Branch::Wu));
    assert_eq!(harm.partner_of(branch), Some(Branch::Wei));
    assert_eq!(breaking.partner_of(branch), Some(Branch::You));
    assert_eq!(SixClash::from_branches([Branch::Wu, branch]), Some(clash));
    assert_eq!(SixHarm::from_branches([Branch::Wei, branch]), Some(harm));
    assert_eq!(
        SixBreak::from_branches([Branch::You, branch]),
        Some(breaking)
    );
    assert_eq!(SixClash::from_branches([branch; 2]), None);
    assert_eq!(SixHarm::from_branches([branch, Branch::Wu]), None);
    assert_eq!(SixBreak::from_branches([branch, Branch::Wei]), None);
    for (label, [a, b]) in [
        ("六冲", clash.members()),
        ("六害", harm.members()),
        ("六壬六破", breaking.members()),
    ] {
        println!("所属{label}配对：{a}、{b}");
    }
    // 巳申同时属于六合及本库六破，刑仍保留参数方向。
    assert_eq!(
        SixCombination::from_branches([Branch::Si, Branch::Shen]),
        Some(SixCombination::SiShen)
    );
    assert_eq!(
        SixBreak::from_branches([Branch::Si, Branch::Shen]),
        Some(SixBreak::SiShen)
    );
    assert!(Branch::Si.is_punishing(Branch::Shen));
    assert!(!Branch::Shen.is_punishing(Branch::Si));
}

fn show_branch_groups(branch: Branch) {
    let combination = branch.three_combination();
    let meeting = branch.three_meeting();
    assert_eq!(combination, ThreeCombination::ShenZiChen);
    assert_eq!(meeting, ThreeMeeting::HaiZiChou);
    assert_eq!(
        combination.members(),
        [Branch::Zi, Branch::Chen, Branch::Shen]
    );
    assert_eq!(meeting.members(), [Branch::Zi, Branch::Chou, Branch::Hai]);
    assert_eq!(
        ThreeCombination::from_branches([Branch::Shen, branch, Branch::Chen]),
        Some(combination),
    );
    assert_eq!(
        ThreeCombination::from_branches([branch, branch, Branch::Shen]),
        None
    );
    for (label, [a, b, c], element) in [
        ("三合组", combination.members(), combination.element()),
        ("三会组", meeting.members(), meeting.element()),
    ] {
        println!(
            "所属{label}：{a}、{b}、{c}；固定对应：{}",
            element_name(element)
        );
    }
}

// Primitive 和 Element 暂无中文 Display；展示文案由示例调用方提供。
fn primitive_name(value: Primitive) -> &'static str {
    match value {
        Primitive::Yang => "阳",
        Primitive::Yin => "阴",
    }
}

fn element_name(value: Element) -> &'static str {
    match value {
        Element::Wood => "木",
        Element::Fire => "火",
        Element::Earth => "土",
        Element::Metal => "金",
        Element::Water => "水",
    }
}
