===description===
@return 2|3 flags return 1 (not in union), but allows return $x after === narrowing
===file===
<?php
/** @param 1|2|3 $x
 *  @return 2|3 */
function strictlyNotOne(int $x): int {
    if ($x === 1) {
        return 1;
    }
    return $x;
}
===expect===
InvalidReturnType@6:8-6:17: Return type '1' is not compatible with declared '2|3'
