===description===
FP: union sub type against union bound - string|SomeCountable satisfies string|Countable
===file===
<?php

interface Countable {}
class MyCountable implements Countable {}

/**
 * @template T of string|Countable
 * @param T $value
 */
function process($value): void {}

$cond = true;
$x = $cond ? 'hello' : new MyCountable();
process($x); // T = string|MyCountable, should pass - each arm satisfies one arm of bound
===expect===
UnusedParam@10:17-10:23: Parameter $value is never used
