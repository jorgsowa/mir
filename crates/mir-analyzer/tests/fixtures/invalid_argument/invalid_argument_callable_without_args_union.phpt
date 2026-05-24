===description===
invalidArgumentCallableWithoutArgsUnion
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
UnusedParam@2:13: Parameter $a is never used
UnusedParam@8:14: Parameter $callable is never used
InvalidArgument@9:5: Argument $callback of typed_callable() expects 'callable with 0 required parameter(s)', got 'callable with 1 required parameter(s)'
