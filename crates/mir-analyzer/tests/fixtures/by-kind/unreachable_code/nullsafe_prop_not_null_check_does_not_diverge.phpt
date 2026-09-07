===description===
`$i?->value !== null` must not be flagged as always-true/false — the
nullsafe access reads null when EITHER the receiver `$i` is null (the
short-circuit) or `value` itself is null, so proving `value`'s own
declared type excludes null does not make the false branch (`$i` is null)
unreachable. narrow_nullsafe_prop_null previously delegated straight to
narrow_prop_null, which (correctly, for the direct non-nullsafe case) now
marks divergence on an impossible property-null comparison — but applied
that same logic unconditionally to the nullsafe case too, producing a
false RedundantCondition here.
===config===
suppress=UnusedVariable,UnusedParam,MissingConstructor
===file===
<?php

class Inner {
    public bool|string $value = true;
}

function doesNotDiverge(?Inner $i): void {
    if ($i?->value !== null) {
        echo "reachable when \$i is non-null";
    } else {
        echo "also reachable when \$i is null";
    }
}
===expect===
