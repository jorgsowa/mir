===description===
Same static-property-read gap as the self::$n case, but reached through
an explicit class name (Counter::$n) rather than self/static/parent --
this path goes through a different helper (record_static_prop_access)
that needed the identical check added separately.
===file===
<?php
class Counter {
    public static int $n = 0;
}

/** @pure */
function peek(): int {
    return Counter::$n;
}
===expect===
ImpureStaticPropertyAccess@8:20-8:22: Reading static property Counter::$n in a @pure function
