===description===
`array_slice`/`array_merge`/`array_push`/`array_unshift`'s list-detection
only checked `TList`/`TNonEmptyList`, missing the `TKeyedArray{is_list:
true}` representation an array literal (`[1, 2, 3]`) actually uses —
operating on a literal-array argument lost its list-ness.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function sliceLiteralListStaysList(): void {
    $r = array_slice([1, 2, 3], 1);
    /** @mir-check $r is list<1|2|3> */
    $_ = $r;
}

function mergeLiteralListsStaysList(): void {
    $r = array_merge([1, 2], ['a', 'b']);
    /** @mir-check $r is non-empty-list<1|2|"a"|"b"> */
    $_ = $r;
}

function pushOntoLiteralListStaysList(): void {
    $arr = [1, 2];
    array_push($arr, 3);
    /** @mir-check $arr is non-empty-list<1|2|3> */
    $_ = $arr;
}

function unshiftOntoLiteralListStaysList(): void {
    $arr = [1, 2];
    array_unshift($arr, 3);
    /** @mir-check $arr is non-empty-list<1|2|3> */
    $_ = $arr;
}
===expect===
