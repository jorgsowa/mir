===description===
Wrong callable return type
===file===
<?php
$add_one = function(int $a): int {
    return $a + 1;
};

/**
 * @param callable(int) : int $c
 */
function bar(callable $c) : string {
    return $c(1);
//  ^^^^^^^^^^^^^ InvalidReturnType: Return type 'int' is not compatible with declared 'string'
}

bar($add_one);
===expect===
