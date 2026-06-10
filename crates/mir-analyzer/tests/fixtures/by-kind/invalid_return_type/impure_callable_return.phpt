===description===
Impure callable return
===ignore===
TODO
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
