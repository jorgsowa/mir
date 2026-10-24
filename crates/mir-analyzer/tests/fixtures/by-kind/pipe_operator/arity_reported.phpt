===description===
`|>` passes exactly one argument, so a callable with a second required
parameter is called with too few arguments.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function add(int $a, int $b): int { return $a + $b; }

$a = 1 |> add(...);
//   ^^^^^^^^^^^^^ TooFewArguments: Too few arguments for add(): expected 2, got 1
$b = 1 |> (fn(int $x, int $y): int => $x + $y);
//   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TooFewArguments: Too few arguments for {closure}(): expected 2, got 1
$c = 1 |> (fn(int $x, int $y = 2): int => $x + $y);
