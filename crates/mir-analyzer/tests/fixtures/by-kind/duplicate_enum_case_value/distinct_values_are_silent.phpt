===description===
Distinct values, a pure enum and a mismatched case type report no duplicate.
===file===
<?php
enum A: int {
    case X = 1;
    case Y = 2;
    case Z = -1;
}

enum B {
    case X;
    case Y;
}

enum C: int {
    case X = 1;
    case Y = '1';
//           ^^^ BackedEnumCaseTypeMismatch: Enum case type string does not match enum backing type int
}
