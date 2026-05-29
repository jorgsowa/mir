===description===
Invalid argument callable without args union
===file===
<?php
function foo(int $a): void {}

/**
 * @param callable()|float $callable
 * @return void
 */
function acme($callable) {}
acme("foo");
===expect===
UnusedParam@2:14-2:20: Parameter $a is never used
UnusedParam@8:15-8:24: Parameter $callable is never used
InvalidArgument@9:6-9:11: Argument $callback of typed_callable() expects 'callable with 0 required parameter(s)', got 'callable with 1 required parameter(s)'
