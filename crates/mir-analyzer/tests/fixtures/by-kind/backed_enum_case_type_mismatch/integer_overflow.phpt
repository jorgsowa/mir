===description===
A case value whose integer arithmetic overflows is a float, which an int-backed enum rejects.
===file===
<?php
enum E: int {
    case A = PHP_INT_MAX + 1;
//           ^^^^^^^^^^^^^^^ BackedEnumCaseTypeMismatch: Enum case type float does not match enum backing type int
    case B = PHP_INT_MIN - 1;
//           ^^^^^^^^^^^^^^^ BackedEnumCaseTypeMismatch: Enum case type float does not match enum backing type int
    case C = 9223372036854775807 * 2;
//           ^^^^^^^^^^^^^^^^^^^^^^^ BackedEnumCaseTypeMismatch: Enum case type float does not match enum backing type int
    case D = -9223372036854775807 - 1;
}
