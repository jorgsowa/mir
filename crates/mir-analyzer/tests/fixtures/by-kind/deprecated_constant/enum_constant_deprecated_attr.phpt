===description===
FN: enum constants never checked the #[Deprecated] attribute fallback,
unlike class constants — only the @deprecated docblock tag worked.
===config===
<mir>
  <issueHandlers>
    <MixedAssignment errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
enum Suit {
    case Hearts;

    #[Deprecated]
    const DEFAULT_SUIT = self::Hearts;
}

$v = Suit::DEFAULT_SUIT;
//         ^^^^^^^^^^^^ DeprecatedConstant: Constant Suit::DEFAULT_SUIT is deprecated
