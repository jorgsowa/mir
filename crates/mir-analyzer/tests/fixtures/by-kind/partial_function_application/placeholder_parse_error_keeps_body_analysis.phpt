===description===
Below PHP 8.6 a placeholder is a `VersionTooLow` parse error, which leaves the AST
complete, so the rest of the call is still checked.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
  <phpVersion>8.5</phpVersion>
</mir>
===file===
<?php

function add(int $a, int $b): int {
    return $a + $b;
}

$partial = add(?, "not-an-int");
//             ^ ParseError: Parse error: 'partial function application' requires PHP 8.6 or higher
//                ^^^^^^^^^^^^ InvalidArgument: Argument $b of add() expects 'int', got '"not-an-int"'
