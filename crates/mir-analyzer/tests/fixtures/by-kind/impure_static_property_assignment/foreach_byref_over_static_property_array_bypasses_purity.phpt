===description===
`foreach (Bag::$queue as &$v)` mutates a static property's array contents
in place, exactly as much as a by-ref call argument
(`array_push(Bag::$queue, 1)`, already checked) does — but the foreach
statement never routed the iterable expression through the same purity
check at all.
===config===
suppress=UnusedForeachValue,MixedAssignment
===file===
<?php
class Bag {
    public static array $queue = [1, 2, 3];
}

/** @pure */
function bumpAll(): void {
    foreach (Bag::$queue as &$v) {
        $v++;
    }
}
===expect===
ImpureStaticPropertyAssignment@8:13-8:24: Assigning to static property Bag::$queue in a @pure function
ImpureStaticPropertyAccess@8:18-8:24: Reading static property Bag::$queue in a @pure function
