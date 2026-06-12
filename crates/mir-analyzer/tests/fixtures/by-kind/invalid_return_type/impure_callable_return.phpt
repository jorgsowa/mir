===description===
Impure callable return
===file===
<?php
/**
 * @pure
 * @return pure-callable():int
 */
function foo(): callable {
    /** @suppress ImpureFunctionCall */
    return function() {
        echo "bar";
        return 1;
    };
}
===expect===
UnusedPsalmSuppress@8:0-8:0: Suppress annotation for 'ImpureFunctionCall' is never used
