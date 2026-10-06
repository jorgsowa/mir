===description===
`ImpureByRefAssignment` only ever fired for a plain `=`/compound-arithmetic
write to a by-reference parameter (`assign_to_target`'s own `Variable`
arm) — every other mutation shape on the same bare variable (`.=`,
`++`/`--`, an array-index write, `unset()` of an array element, a by-ref
`foreach`, or passing it further by reference to a builtin like `sort()`)
silently bypassed the check entirely, unlike the sibling property-receiver
case which already covers all of these shapes.
===config===
<mir>
  <issueHandlers>
    <MissingReturnType errorLevel="suppress"/>
    <UnusedForeachValue errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
    <ImpureFunctionCall errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @pure */
function concatByRef(string &$s): void {
    $s .= 'x';
//  ^^^^^^^^^ ImpureByRefAssignment: Assigning to by-reference parameter $s in a @pure function
}

/** @pure */
function incByRef(int &$n): void {
    $n++;
//  ^^ ImpureByRefAssignment: Assigning to by-reference parameter $n in a @pure function
}

/** @pure */
function arrWriteByRef(array &$arr): void {
    $arr['k'] = 1;
//  ^^^^^^^^^^^^^ ImpureByRefAssignment: Assigning to by-reference parameter $arr in a @pure function
}

/** @pure */
function unsetByRef(array &$arr): void {
    unset($arr['k']);
//        ^^^^^^^^^ ImpureByRefAssignment: Assigning to by-reference parameter $arr in a @pure function
}

/** @pure */
function foreachByRef(array &$arr): void {
    foreach ($arr as &$v) {
//           ^^^^ ImpureByRefAssignment: Assigning to by-reference parameter $arr in a @pure function
        $v = 1;
    }
}

/** @pure */
function sortByRef(array &$arr): void {
    sort($arr);
//       ^^^^ ImpureByRefAssignment: Assigning to by-reference parameter $arr in a @pure function
}
