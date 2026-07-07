===description===
`$param is not X ? A : B` must parse as sugar for `$param is X ? B : A` — the
negated form PHPStan/Psalm both support, previously unrecognized by the
conditional-type parser (which only matched the bare `is` form) so the whole
`@return` tag silently failed to parse into a conditional type.
===config===
suppress=UnusedVariable
===file===
<?php
/**
 * @param mixed $value
 * @return ($value is not string ? int : string)
 */
function classify($value) {
    if (is_string($value)) {
        return "text";
    }
    return 1;
}

$a = classify(1);
/** @mir-check $a is int */
echo "a";

$b = classify("s");
/** @mir-check $b is string */
echo "b";
===expect===
