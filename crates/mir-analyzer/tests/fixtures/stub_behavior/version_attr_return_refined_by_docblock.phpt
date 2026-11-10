===description===
A builtin whose `#[LanguageLevelTypeAware]` return type is plain `array` keeps
its docblock's element type, minus the `|false` PHP 8 no longer returns, so
`mb_str_split()` returns strings, not `mixed`.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function takesString(string $value): void {}

function split(string $s): void {
    $parts = mb_str_split($s, 2);
    /** @mir-check $parts is array<int|string, string> */
    takesString($parts[0]);
}
