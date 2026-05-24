===description===
closureInvalidArg
===file===
<?php
/** @param Closure(int): string $c */
function takesClosure(Closure $c): void {}

takesClosure(5);
===expect===
UnusedParam@3:23: Parameter $c is never used
InvalidArgument@5:14: Argument $c of takesClosure() expects 'Closure(int): string', got '5'
