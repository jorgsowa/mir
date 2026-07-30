===description===
Negative control for the optional-trailing-param arity fix: a closure requiring MORE
params than the FULL declared signature (including the `=`-marked ones) is still unsafe —
the type never promises more than that many args will ever be passed — and must still flag.
===config===
suppress=UnusedParam
===file===
<?php
/** @param Closure(mixed, array=, string=):mixed $cb */
function register(Closure $cb): void {}

register(function (int $a, array $b, string $c, string $d): void {});

===expect===
InvalidArgument@5:9-5:67: Argument $cb of register() expects 'callable with 3 required parameter(s)', got 'callable with 4 required parameter(s)'
