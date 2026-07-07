===description===
An outer plain `is` conditional whose false branch nests a negated `is not`
conditional must split on the OUTER marker, not whichever `is`/`is not`
appears anywhere in the string. The marker search must be depth-aware: a
plain substring search for ` is not ` finds the nested one first (it's
prioritized unconditionally) even though the outer marker comes first in the
source text, corrupting the outer split (garbage param name, mismatched
branches).
===file===
<?php
/**
 * @param mixed $value
 * @return ($value is string ? int : ($value is not array ? bool : string))
 */
function classify($value) {
    if (is_string($value)) {
        return 1;
    }
    if (!is_array($value)) {
        return true;
    }
    return "arr";
}

$a = classify('s');
/** @mir-check $a is int */
echo $a;

$b = classify(true);
/** @mir-check $b is bool */
echo $b;

$c = classify([1, 2]);
/** @mir-check $c is string */
echo $c;
===expect===
