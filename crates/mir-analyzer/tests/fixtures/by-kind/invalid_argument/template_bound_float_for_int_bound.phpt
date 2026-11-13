===description===
A float never holds an int, so a float for a template bound by `int` is an invalid argument, not a coercion.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @template T of int
 * @param T $n
 */
function step($n): void {}

function run(float $ratio): void {
    step(2);
    step($ratio);
//  ^^^^^^^^^^^^ InvalidArgument: Argument $n of step() expects 'int', got 'float'
}
