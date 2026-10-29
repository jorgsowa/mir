===description===
A `callable-string` names a function, so it satisfies a `callable` parameter as a plain `string` does; other parts of the argument are still checked.
===file===
<?php
/** @param callable $c */
function take($c): void { $c(); }

/** @param callable(int): void $c */
function takeTyped($c): void { $c(1); }

/** @param callable-string $a */
function only($a): void { take($a); takeTyped($a); }

/** @param string|callable-string $a */
function withString($a): void { take($a); }

/** @param callable-string|callable $a */
function withCallable($a): void { take($a); }

/** @param callable-string|int $a */
function withInt($a): void { take($a); }
//                                ^^ PossiblyInvalidArgument: Argument $c of take() expects 'callable', possibly different type 'callable-string|int' provided
