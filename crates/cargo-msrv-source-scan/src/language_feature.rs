use std::fmt;

/// A language feature which requires at least a certain Rust version.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum LanguageFeature {
    /// `x?`
    QuestionMarkOperator,
    /// `fn f(x: impl Trait)` and `fn f() -> impl Trait`
    ImplTrait,
    /// `dyn Trait`
    DynTrait,
    /// `async fn`, `async {}`, and `x.await`
    AsyncAwait,
    /// `struct S<const N: usize>`
    ConstGenerics,
    /// `X..` patterns
    HalfOpenRangePattern,
    /// `format!("{x}")`
    ImplicitFormatArguments,
    /// `let Some(x) = y else { return };`
    LetElse,
    /// `type Item<'a>;` in a trait
    GenericAssociatedTypes,
    /// `'label: { break 'label; }`
    LabeledBlock,
    /// `async fn` and `-> impl Trait` in a trait
    AsyncFnAndImplTraitInTrait,
    /// `c"hello"`
    CStringLiteral,
    /// `#[diagnostic::on_unimplemented]`
    DiagnosticAttributeNamespace,
    /// `const { ... }` in an expression
    InlineConst,
    /// `X..Y` patterns
    ExclusiveRangePattern,
    /// `#[expect(lint)]`
    ExpectLintAttribute,
    /// `&raw const x` and `&raw mut x`
    RawReference,
    /// `async || {}`
    AsyncClosure,
    /// `if let Some(x) = y && x > 0`
    LetChains,
}

impl LanguageFeature {
    /// The Rust version in which the feature was stabilized.
    pub fn since(self) -> semver::Version {
        let minor = match self {
            Self::QuestionMarkOperator => 13,
            Self::ImplTrait => 26,
            Self::DynTrait => 27,
            Self::AsyncAwait => 39,
            Self::ConstGenerics => 51,
            Self::HalfOpenRangePattern => 55,
            Self::ImplicitFormatArguments => 58,
            Self::LetElse | Self::GenericAssociatedTypes | Self::LabeledBlock => 65,
            Self::AsyncFnAndImplTraitInTrait => 75,
            Self::CStringLiteral => 77,
            Self::DiagnosticAttributeNamespace => 78,
            Self::InlineConst => 79,
            Self::ExclusiveRangePattern => 80,
            Self::ExpectLintAttribute => 81,
            Self::RawReference => 82,
            Self::AsyncClosure => 85,
            Self::LetChains => 88,
        };

        semver::Version::new(1, minor, 0)
    }

    fn description(self) -> &'static str {
        match self {
            Self::QuestionMarkOperator => "the `?` operator",
            Self::ImplTrait => "`impl Trait`",
            Self::DynTrait => "`dyn Trait`",
            Self::AsyncAwait => "async/await",
            Self::ConstGenerics => "const generics",
            Self::HalfOpenRangePattern => "half-open range patterns",
            Self::ImplicitFormatArguments => "implicit format arguments",
            Self::LetElse => "let-else statements",
            Self::GenericAssociatedTypes => "generic associated types",
            Self::LabeledBlock => "labeled blocks",
            Self::AsyncFnAndImplTraitInTrait => "`async fn` or `-> impl Trait` in a trait",
            Self::CStringLiteral => "C string literals",
            Self::DiagnosticAttributeNamespace => "the `#[diagnostic]` attribute namespace",
            Self::InlineConst => "inline const expressions",
            Self::ExclusiveRangePattern => "exclusive range patterns",
            Self::ExpectLintAttribute => "the `#[expect]` lint attribute",
            Self::RawReference => "raw references (`&raw const` and `&raw mut`)",
            Self::AsyncClosure => "async closures",
            Self::LetChains => "let chains",
        }
    }
}

impl fmt::Display for LanguageFeature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.description())
    }
}
