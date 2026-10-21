===description===
Arguments of a partial application are checked against the callee's parameters.
===config===
<mir>
  <phpVersion>8.6</phpVersion>
</mir>
===file===
<?php
function add(int $a, int $b): int { return $a + $b; }

function f(): void {
    add('x', ?);
    add(?);
    add(?, ?, ?);
    add(1, ?);
    add(?, ?);
}
===expect===
InvalidArgument@5:8-5:11: Argument $a of add() expects 'int', got '"x"'
TooFewArguments@6:4-6:10: Too few arguments for add(): expected 2, got 1
TooManyArguments@7:14-7:15: Too many arguments for add(): expected 2, got 3
