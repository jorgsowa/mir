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
//      ^^^ InvalidArgument: Argument $a of add() expects 'int', got '"x"'
    add(?);
//  ^^^^^^ TooFewArguments: Too few arguments for add(): expected 2, got 1
    add(?, ?, ?);
//            ^ TooManyArguments: Too many arguments for add(): expected 2, got 3
    add(1, ?);
    add(?, ?);
}
