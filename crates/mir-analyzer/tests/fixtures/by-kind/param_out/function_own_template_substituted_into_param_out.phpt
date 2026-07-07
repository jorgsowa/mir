===description===
A plain function's own `@template` bound from its `@param` must also be
substituted into its own `@param-out` type. The by-ref write-back loop ran
before the function's inferred template bindings were computed, so a
generic identity/setter-style function always wrote back the raw
`TTemplateParam` atom instead of the concrete argument type.
===config===
suppress=UnusedVariable
===file===
<?php
/**
 * @template T
 * @param T $in
 * @param-out T $out
 */
function identity($in, mixed &$out): void {
    $out = $in;
}

identity(42, $result);
/** @mir-check $result is int */
$_ = $result;
===expect===
