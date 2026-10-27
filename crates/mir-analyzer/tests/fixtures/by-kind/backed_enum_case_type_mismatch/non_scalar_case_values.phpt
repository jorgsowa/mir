===description===
PHP names the value's kind: `object` for an enum case or other object, `array`, `null`, `bool`, `float`.
===file===
<?php
enum O {
    case P;
}

class K {
    const ARR = [1];
    const F = 1.5;
}

enum E: int {
    case B = 5;
    case A = O::P;
//           ^^^^ BackedEnumCaseTypeMismatch: Enum case type object does not match enum backing type int
    case C = self::B;
//           ^^^^^^^ BackedEnumCaseTypeMismatch: Enum case type object does not match enum backing type int
    case D = K::ARR;
//           ^^^^^^ BackedEnumCaseTypeMismatch: Enum case type array does not match enum backing type int
    case F = K::F;
//           ^^^^ BackedEnumCaseTypeMismatch: Enum case type float does not match enum backing type int
    case G = !0;
//           ^^ BackedEnumCaseTypeMismatch: Enum case type bool does not match enum backing type int
    case H = null;
//           ^^^^ BackedEnumCaseTypeMismatch: Enum case type null does not match enum backing type int
}
