===description===
A return type that grows on every fixpoint iteration (`[$this->f()]`,
`n() + 1`) widens to `mixed` instead of diverging.
===config===
suppress=MissingReturnType,UnusedVariable
===file===
<?php
class T {
    function f() { return [$this->f()]; }
}
function n() { return n() + 1; }

/** @mir-check $a is mixed */
$a = (new T)->f();
/** @mir-check $b is mixed */
$b = n();
===expect===
MixedAssignment@8:0-8:17: Variable $a is assigned a mixed type
MixedAssignment@10:0-10:8: Variable $b is assigned a mixed type
