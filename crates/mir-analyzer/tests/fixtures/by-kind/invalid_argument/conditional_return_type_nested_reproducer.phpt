===description===
nested conditional return type passed to typed param should not report InvalidArgument
===file===
<?php
/**
 * @template TKey of array-key
 * @template TValue
 * @param TValue|array<TKey, TValue>|null $value
 * @return ($value is null ? array{} : ($value is array ? array<TKey, TValue> : array{TValue}))
 */
function wrap($value) { return is_null($value) ? [] : (is_array($value) ? $value : [$value]); }

/** @param array<mixed, mixed> $a */
function takesArray(array $a): void { var_dump($a); }

$x = 'hello';
takesArray(wrap($x));
takesArray(wrap(null));
takesArray(wrap(['a', 'b']));
===expect===
