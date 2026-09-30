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
//  ^^^^^^^^^^^ Trace: Type of $a is string|null
//         ^^ PossiblyNullArgument: Argument $string of strlen() might be null
}
===expect===
