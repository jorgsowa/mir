===description===
Psalm pseudo-types in a top-level function @return are not reported as undefined classes
===file===
<?php
/** @return truthy-string */
function label() {
    return "x";
}

/** @return int-mask<1, 2, 4> */
function flags() {
    return 3;
}

/** @return non-falsy-string */
function name() {
    return "n";
}

===expect===
