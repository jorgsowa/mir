===description===
A bounded int range that lies entirely within a named int subtype's domain
is accepted as a valid return — and one that falls outside is rejected.
===file===
<?php

/** @return positive-int */
function ok_pos(): int {
    /** @var int<1, 100> $x */
    $x = 42;
    return $x;
}

/** @return non-negative-int */
function ok_nonneg(): int {
    /** @var int<0, 50> $x */
    $x = 0;
    return $x;
}

/** @return negative-int */
function ok_neg(): int {
    /** @var int<-10, -1> $x */
    $x = -5;
    return $x;
}

/** @return positive-int */
function bad_pos(): int {
    /** @var int<0, 10> $x */
    $x = 0;
    return $x;
}
===expect===
InvalidReturnType@28:4-28:14: Return type 'int<0, 10>' is not compatible with declared 'positive-int'
