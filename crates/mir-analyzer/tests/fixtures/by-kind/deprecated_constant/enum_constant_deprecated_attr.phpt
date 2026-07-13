===description===
FN: enum constants never checked the #[Deprecated] attribute fallback,
unlike class constants — only the @deprecated docblock tag worked.
===config===
suppress=MixedAssignment,UnusedVariable
===file===
<?php
enum Suit {
    case Hearts;

    #[Deprecated]
    const DEFAULT_SUIT = self::Hearts;
}

$v = Suit::DEFAULT_SUIT;
===expect===
DeprecatedConstant@9:11-9:23: Constant Suit::DEFAULT_SUIT is deprecated
