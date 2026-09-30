===description===
Invalid argument callable without args union
===file===
<?php
function foo(int $a): void {}
//           ^^^^^^ UnusedParam: Parameter $a is never used

/**
 * @param callable()|float $callable
 * @return void
 */
function acme($callable) {}
//            ^^^^^^^^^ UnusedParam: Parameter $callable is never used
acme("foo");
//   ^^^^^ InvalidArgument: Argument $callable of acme() expects 'callable with 0 required parameter(s)', got 'callable with 1 required parameter(s)'
===expect===
