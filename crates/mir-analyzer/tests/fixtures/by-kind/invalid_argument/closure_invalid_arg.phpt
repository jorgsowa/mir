===description===
Closure invalid arg
===file===
<?php
/** @param Closure(int): string $c */
function takesClosure(Closure $c): void {}
//                    ^^^^^^^^^^ UnusedParam: Parameter $c is never used

takesClosure(5);
//           ^ InvalidArgument: Argument $c of takesClosure() expects 'Closure(int): string', got '5'
