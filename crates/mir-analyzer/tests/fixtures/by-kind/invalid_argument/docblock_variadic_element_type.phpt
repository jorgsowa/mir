===description===
@param Type ...$name on a variadic parameter is parsed and checked at call sites
===file===
<?php
/** @param int ...$nums */
function sumAll(...$nums): int {
    return array_sum($nums);
}

sumAll("a", "b");
//     ^^^ InvalidArgument: Argument $nums of sumAll() expects 'int', got '"a"'
//          ^^^ InvalidArgument: Argument $nums of sumAll() expects 'int', got '"b"'
===expect===
