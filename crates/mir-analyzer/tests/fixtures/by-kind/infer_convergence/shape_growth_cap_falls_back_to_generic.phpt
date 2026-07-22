===description===
A shape can only grow up to a fixed number of properties from straight-line
literal writes (MAX_SHAPE_KEYS = 8). The write that would exceed the cap
generalizes the whole variable to a plain `list<int>` instead, the same
fallback a loop would produce — this keeps a long run of literal pushes from
producing an ever-growing printed shape.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param list<int> $arr */
function test(array $arr): void {
    if ($arr === []) {
        $arr[] = 1;
        $arr[] = 2;
        $arr[] = 3;
        $arr[] = 4;
        $arr[] = 5;
        $arr[] = 6;
        $arr[] = 7;
        $arr[] = 8;
        $arr[] = 9;
        /** @mir-check $arr is list<int> */
        $_ = $arr;
    }
}
===expect===
