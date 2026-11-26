use matharts_core::{Branch, CyclicSequence, Ganzhi, ParseError};

fn main() -> Result<(), ParseError> {
    let value: Ganzhi = "甲子".parse()?;
    assert_eq!(value.offset(1).to_string(), "乙丑");
    assert_eq!(value.offset(60), value);
    assert_eq!(value.xun().void_branches(), (Branch::Xu, Branch::Hai));
    assert_eq!(value.nayin().name(), "海中金");
    assert!("甲丑".parse::<Ganzhi>().is_err());

    #[cfg(feature = "serde")]
    {
        let wire = r#"{"stem":"Jia","branch":"Zi"}"#;
        assert_eq!(serde_json::to_string(&value).unwrap(), wire);
        assert_eq!(serde_json::from_str::<Ganzhi>(wire).unwrap(), value);
        assert!(serde_json::from_str::<Ganzhi>(r#"{"stem":"Jia","branch":"Chou"}"#).is_err());
    }

    Ok(())
}
