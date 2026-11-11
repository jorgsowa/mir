===description===
A returned value is checked against a mask over another class; a mask over an unknown class stays `int`.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
final class Style {
    public const BOLD = 1;
    public const ITALIC = 2;
}

/** @return int-mask-of<Style::*> */
function heading(): int {
    return Style::BOLD | Style::ITALIC;
}

/** @return int-mask-of<Style::*> */
function broken(): int {
    return 16;
//  ^^^^^^^^^^ InvalidReturnType: Return type '16' is not compatible with declared 'int-mask-of<Style::*>'
}

/** @param int-mask-of<Vendor\Missing::*> $flags */
function external(int $flags): void {}

external(1024);
external('none');
//       ^^^^^^ InvalidArgument: Argument $flags of external() expects 'int', got '"none"'
