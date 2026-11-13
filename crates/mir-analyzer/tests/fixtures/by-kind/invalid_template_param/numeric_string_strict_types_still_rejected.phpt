===description===
Strict types keep rejecting numeric strings for an int|float template bound.
===file===
<?php
declare(strict_types=1);

/**
 * @template T of int|float
 * @param T $n
 * @return T
 */
function twice($n) { return $n; }

/** @param numeric-string $ns */
function f($ns): void {
    echo twice($ns);
//       ^^^^^^^^^^ InvalidArgument: Argument $n of twice() expects 'int|float', got 'numeric-string'
}
