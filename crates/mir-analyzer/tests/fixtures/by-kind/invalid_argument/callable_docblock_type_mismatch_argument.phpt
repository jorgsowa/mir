===description===
A bare docblock callable(T):R type (not wrapped in Closure/an intersection)
now gets full argument-type checking, not just arity.
===file===
<?php
/**
 * @param callable(int):void $fn
 */
function apply(callable $fn): void {
    $fn('not an int');
}
===expect===
InvalidArgument@6:8-6:20: Argument $arg0 of callable() expects 'int', got '"not an int"'
