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
UndefinedDocblockClass@6:10-6:13: Docblock type 'pure-callable():int' does not exist
UnusedPsalmSuppress@8:0-8:0: Suppress annotation for 'ImpureFunctionCall' is never used
