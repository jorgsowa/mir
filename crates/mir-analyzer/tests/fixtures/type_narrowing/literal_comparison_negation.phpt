===description===
literal comparison negation: !== removes the specific literal from a union
===file===
<?php
/** @param 1|2|3 $x */
function removeOne(int $x): int {
    if ($x !== 1) {
        return $x;
    }
    return 1;
}

/** @param "foo"|"bar"|"baz" $x */
function removeString(string $x): string {
    if ($x !== "foo") {
        return $x;
    }
    return "foo";
}

/** @param 1|2|3 $x
 *  @return 2|3 */
function strictlyNotOne(int $x): int {
    if ($x === 1) {
        return 1;
    }
    return $x;
}
===expect===
InvalidReturnType@22:8: Return type '1' is not compatible with declared '2|3'
