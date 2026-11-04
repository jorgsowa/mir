===description===
An omitted `@param T $x = null` binds T to null, so `(T is null ? A : B)` resolves to A
===file===
<?php
/**
 * @template T of string|null
 * @param T $key
 * @return (T is null ? int : string)
 */
function pick(?string $key = null): int|string { return $key === null ? 1 : $key; }

$a = pick();
/** @mir-check $a is int */
echo $a;
