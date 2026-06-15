===description===
Closure invalid arg
===file===
<?php
/** @param Closure(int): string $c */
function takesClosure(Closure $c): void {}

takesClosure(5);
===expect===
UnusedParam@3:22-3:32: Parameter $c is never used
InvalidArgument@5:13-5:14: Argument $c of takesClosure() expects 'Closure(int): string', got '5'
