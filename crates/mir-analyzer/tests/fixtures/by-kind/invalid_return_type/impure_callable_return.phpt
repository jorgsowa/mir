===description===
Impure callable return
===config===
suppress=MissingClosureReturnType
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
UnusedSuppress@8:0-8:0: Suppress annotation for 'ImpureFunctionCall' is never used
