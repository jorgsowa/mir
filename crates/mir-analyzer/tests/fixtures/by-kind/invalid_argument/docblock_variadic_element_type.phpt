===description===
@param Type ...$name on a variadic parameter is parsed and checked at call sites
===file===
<?php
/** @param int ...$nums */
function sumAll(...$nums): int {
    return array_sum($nums);
}

sumAll("a", "b");
===expect===
InvalidArgument@7:7-7:10: Argument $nums of sumAll() expects 'int', got '"a"'
InvalidArgument@7:12-7:15: Argument $nums of sumAll() expects 'int', got '"b"'
