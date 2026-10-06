===description===
A placeholder argument through a dynamic callable call (`$fn(?, 5)`) exercises
the separate "callee is not a plain identifier" branch of function-call
analysis — must not crash there either.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
  <phpVersion>8.5</phpVersion>
</mir>
===file===
<?php

$fn = function (int $a, int $b): int {
    return $a + $b;
};
$partial = $fn(?, 5);
//             ^ ParseError: Parse error: 'partial function application' requires PHP 8.6 or higher
