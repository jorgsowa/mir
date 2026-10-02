===description===
A subject narrowed to a subset of enum cases only needs those cases covered,
and a missing one among them is still reported.
===file===
<?php
enum Err { case NotFound; case Denied; case Timeout; }

function covered(Err $e): int {
    if ($e === Err::Timeout) {
        return 0;
    }
    /** @mir-check $e is Err::NotFound|Err::Denied */
    return match ($e) {
        Err::NotFound => 1,
        Err::Denied => 2,
    };
}

function missing(Err $e): int {
    if ($e === Err::Timeout) {
        return 0;
    }
    return match ($e) {
        Err::NotFound => 1,
    };
}
===expect===
UnhandledMatchCondition@19:11-21:5: Unhandled match condition: Err::Denied
