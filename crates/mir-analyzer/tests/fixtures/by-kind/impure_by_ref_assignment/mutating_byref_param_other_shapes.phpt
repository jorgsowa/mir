===description===
`ImpureByRefAssignment` only ever fired for a plain `=`/compound-arithmetic
write to a by-reference parameter (`assign_to_target`'s own `Variable`
arm) — every other mutation shape on the same bare variable (`.=`,
`++`/`--`, an array-index write, `unset()` of an array element, a by-ref
`foreach`, or passing it further by reference to a builtin like `sort()`)
silently bypassed the check entirely, unlike the sibling property-receiver
case which already covers all of these shapes.
===config===
suppress=MissingReturnType,UnusedForeachValue,MixedAssignment,UnusedParam,ImpureFunctionCall
===file===
<?php
/** @pure */
function concatByRef(string &$s): void {
    $s .= 'x';
}

/** @pure */
function incByRef(int &$n): void {
    $n++;
}

/** @pure */
function arrWriteByRef(array &$arr): void {
    $arr['k'] = 1;
}

/** @pure */
function unsetByRef(array &$arr): void {
    unset($arr['k']);
}

/** @pure */
function foreachByRef(array &$arr): void {
    foreach ($arr as &$v) {
        $v = 1;
    }
}

/** @pure */
function sortByRef(array &$arr): void {
    sort($arr);
}
===expect===
ImpureByRefAssignment@4:4-4:13: Assigning to by-reference parameter $s in a @pure function
ImpureByRefAssignment@9:4-9:6: Assigning to by-reference parameter $n in a @pure function
ImpureByRefAssignment@14:4-14:17: Assigning to by-reference parameter $arr in a @pure function
ImpureByRefAssignment@19:10-19:19: Assigning to by-reference parameter $arr in a @pure function
ImpureByRefAssignment@24:13-24:17: Assigning to by-reference parameter $arr in a @pure function
ImpureByRefAssignment@31:9-31:13: Assigning to by-reference parameter $arr in a @pure function
