===description===
Negative control: a trailing variadic in the declared callable type (`...string`) promises
zero-or-more extra args, never a guaranteed count — it must stay excluded from the allowed
required-param ceiling, not silently raise it. A closure requiring more fixed params than the
non-variadic prefix must still flag.
===config===
suppress=UnusedParam
===file===
<?php
/** @param Closure(int, ...string):void $cb */
function register(Closure $cb): void {}

register(function (int $a, string $b): void {});

===expect===
InvalidArgument@5:9-5:46: Argument $cb of register() expects 'callable with 1 required parameter(s)', got 'callable with 2 required parameter(s)'
