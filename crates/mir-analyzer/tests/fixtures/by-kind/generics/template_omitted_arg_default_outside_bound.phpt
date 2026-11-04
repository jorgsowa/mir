===description===
An omitted argument whose default is outside the template's bound leaves T at its bound
===file===
<?php
/**
 * @template T of string
 * @param T $key
 * @return T
 */
function one(?string $key = null) { return $key ?? ''; }

$a = one();
/** @mir-check $a is string */
echo $a;
