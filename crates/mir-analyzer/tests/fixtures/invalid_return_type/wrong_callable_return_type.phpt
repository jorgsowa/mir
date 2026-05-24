===description===
wrongCallableReturnType
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
InvalidReturnStatement
===ignore===
TODO
