===description===
A spread argument (...$args) against an intersection-typed callable value
(object&Closure(...)) is not miscounted as too few arguments — the arity-only
fallback used when full param types aren't resolvable must treat a spread
arg as unknown-count, same as the full check_args path already does.
===config===
suppress=MixedArgument,MixedAssignment
===file===
<?php
/**
 * @param object&Closure(int,int,int):void $fn
 */
function test(object $fn): void {
    $args = [1, 2, 3];
    $fn(...$args);
}
===expect===
