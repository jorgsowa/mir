===description===
closureInvalidArg
===file===
<?php
/** @param Closure(int): string $c */
function takesClosure(Closure $c): void {}

takesClosure(5);
===expect===
UnusedParam@3:22: Parameter $c is never used
InvalidArgument@5:13: Argument $c of takesClosure() expects 'Closure(int): string', got '5'
