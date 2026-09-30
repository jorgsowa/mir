===description===
`!isset($x) || class_exists(Undefined::class)`'s speculative RHS evaluation
must not leak the class_exists guard into the merged true-branch: the branch
is also reachable via the "$x unset" path, where class_exists() never ran.
===config===
suppress=UnusedVariable,PossiblyUndefinedVariable,MissingParamType
===file===
<?php
function f($x): void {
    if (!isset($x) || class_exists(\Totally\Undefined\GuardLeak::class)) {
        new \Totally\Undefined\GuardLeak();
//          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UndefinedClass: Class Totally\Undefined\GuardLeak does not exist
    }
    new \Totally\Undefined\GuardLeak();
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UndefinedClass: Class Totally\Undefined\GuardLeak does not exist
}
===expect===
