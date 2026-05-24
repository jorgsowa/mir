===description===
!== removes a specific literal from a union, narrowed type is accepted as more-specific return
===file===
<?php
/** @param 1|2|3 $x
 *  @return 2|3 */
function removeOne(int $x): int {
    if ($x !== 1) {
        return $x;
    }
    return 2;
}

/** @param "foo"|"bar"|"baz" $x
 *  @return "bar"|"baz" */
function removeString(string $x): string {
    if ($x !== "foo") {
        return $x;
    }
    return "bar";
}
===expect===
