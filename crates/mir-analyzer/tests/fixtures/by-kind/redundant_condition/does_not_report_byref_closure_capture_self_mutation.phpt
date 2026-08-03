===description===
M10: a by-ref-captured variable (`use (&$flag)`) is the SAME variable across
every invocation of the closure, including any prior invocation that
mutated it — seeding it with the exact literal type snapshotted at the
closure-literal site (`false`) falsely claims it can never be true, firing
a bogus RedundantCondition. A by-value capture of the same variable keeps
its precise literal type, since it can never be observed after the
closure-literal site.
===config===
suppress=UnusedVariable,MissingClosureReturnType
===file===
<?php
$flag = false;
$cb = static function (string $h) use (&$flag) {
    if ($flag) {
        echo "was set\n";
    }
    if ($h === 'x') {
        $flag = true;
    }
};

$flag2 = false;
$cb2 = static function () use ($flag2) {
    if ($flag2) {
        echo "never\n";
    }
};
===expect===
RedundantCondition@14:8-14:14: Condition is always true/false for type 'false'
