===description===
Reading a static property inside a @pure function was never checked at
all -- asymmetric with static property WRITES (ImpureStaticPropertyAssignment,
already checked) and with superglobal reads (already checked). Reading
shared external state is just as non-deterministic across calls as
writing it.
===file===
<?php
class Counter {
    private static int $n = 0;

    /** @pure */
    public static function peek(): int {
        return self::$n;
    }
}
===expect===
ImpureStaticPropertyAccess@7:21-7:23: Reading static property Counter::$n in a @pure function
