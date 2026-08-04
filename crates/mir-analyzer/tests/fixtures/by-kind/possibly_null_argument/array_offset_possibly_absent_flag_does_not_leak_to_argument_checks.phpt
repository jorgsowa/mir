===description===
L3 boundary check: `possibly_absent_offset` is a narrow provenance flag
consulted only by the identical-comparison/narrowing-divergence checks — it
must NOT add `null` to the type's own member set, so passing the same
generic-array read directly into a non-nullable parameter (no defensive
null check at all) still gets the ordinary, unrelated diagnostic behavior:
none here, since `strtoupper` genuinely only requires `string`, and the
declared value type of `array<int, string>` already satisfies that without
needing `null` added to it.
===config===
suppress=UnusedParam
===file===
<?php
/** @param array<int, string> $map */
function test(array $map): string {
    return strtoupper($map[5]);
}
===expect===
