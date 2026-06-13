===description===
nested conditional return type passed to typed param should not report InvalidArgument
===config===
suppress=ForbiddenCode
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

// Null arg: outer conditional resolves to if_true branch.
$null_result = wrap(null);
/** @mir-check $null_result is array{} */
takesArray($null_result);

// String arg: outer resolves to if_false, inner resolves to if_false — no TConditional survives.
$str_result = wrap('hello');
/** @mir-check $str_result is array{0: "hello"} */
takesArray($str_result);

// Array arg: outer resolves to if_false, inner resolves to if_true — no TConditional survives.
$arr_result = wrap(['a', 'b']);
/** @mir-check $arr_result is array<int, array{0: "a", 1: "b"}|"a"|"b"> */
takesArray($arr_result);

// Union arg (string|null): outer can't resolve, so both branches are widened.
// Recursive widening must also flatten the inner TConditional inside if_false.
/** @var string|null $str_or_null */
$str_or_null = rand() ? 'x' : null;
$union_result = wrap($str_or_null);
/** @mir-check $union_result is array{}|array{0: string} */
takesArray($union_result);

// Flat single-level conditional: widening (subject not in predicate list) collapses to string.
// Regression guard: the fix must not break flat conditionals.
/**
 * @param string $value
 * @return ($value is "" ? "" : string)
 */
function studlyFlat(string $value): string { return $value; }
$flat_result = studlyFlat('hello');
/** @mir-check $flat_result is string */
echo $flat_result;

// 3-level nested: recursion must go all the way down — each distinct arg resolves a different leaf.
// Array return types survive widen_for_check so each branch is distinguishable.
/**
 * @param mixed $a
 * @return ($a is null ? array{} : ($a is string ? array{string} : array<string, string>))
 */
function branch3($a) { return is_null($a) ? [] : (is_string($a) ? [$a] : ['k' => $a]); }
$b_null = branch3(null);
/** @mir-check $b_null is array{} */
$b_str = branch3('x');
/** @mir-check $b_str is array{string} */
$b_other = branch3(42);
/** @mir-check $b_other is array<string, string> */
echo count($b_null) + count($b_str) + count($b_other);
===expect===
