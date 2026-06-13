===description===
array_filter without a callback keeps the value type but drops list-ness (gaps allowed)
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param list<int> $nums */
function test(array $nums): void {
    $r = array_filter($nums);
    /** @mir-check $r is array<int, int> */
    $_ = $r;
}
===expect===
