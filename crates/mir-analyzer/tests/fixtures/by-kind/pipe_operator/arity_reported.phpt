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
$b = 1 |> (fn(int $x, int $y): int => $x + $y);
$c = 1 |> (fn(int $x, int $y = 2): int => $x + $y);
===expect===
TooFewArguments@4:5-4:18: Too few arguments for add(): expected 2, got 1
TooFewArguments@5:5-5:46: Too few arguments for {closure}(): expected 2, got 1
