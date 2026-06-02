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
}

bar($add_one);
===expect===
InvalidReturnType@10:5-10:18: Return type 'int' is not compatible with declared 'string'
