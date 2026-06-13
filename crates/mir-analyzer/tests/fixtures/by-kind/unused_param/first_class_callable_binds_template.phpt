===description===
first-class callable syntax (strlen(...)) produces a typed TClosure so T binds correctly
===config===
suppress=MixedArgument
===file===
<?php
/**
 * @template T
 * @template R
 * @param Closure(T): R $fn
 * @param T $value
 * @return R
 */
function apply(callable $fn, mixed $value): mixed { return $fn($value); }

$result = apply(strlen(...), 'hello');
/** @mir-check $result is int */
echo $result;
===expect===
