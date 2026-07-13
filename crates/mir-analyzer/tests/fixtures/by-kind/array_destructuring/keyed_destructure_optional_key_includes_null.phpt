===description===
An optional shape key (`array{a?: T}`) destructured via `['a' => $a] = $arr`
must widen $a's type with null, same as plain array access ($arr['a']).
===config===
suppress=UnusedVariable
===file===
<?php
/**
 * @param array{a?: string} $arr
 */
function test(array $arr): void {
    ['a' => $a] = $arr;
    /** @trace $a */
    strlen($a);
}
===expect===
Trace@8:4-8:15: Type of $a is string|null
PossiblyNullArgument@8:11-8:13: Argument $string of strlen() might be null
