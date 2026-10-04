===description===
Only `+` accepts arrays; other arithmetic operators with an array and a
`mixed` operand are always a TypeError.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
    <MissingThrowsDocblock errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function sub(mixed $m): void {
    $a = [1] - $m;
//       ^^^^^^^^ InvalidOperand: Operator '-' not supported between 'array{0: 1}' and 'mixed'
    $b = $m * [1];
//       ^^^^^^^^ InvalidOperand: Operator '*' not supported between 'mixed' and 'array{0: 1}'
}
===expect===
