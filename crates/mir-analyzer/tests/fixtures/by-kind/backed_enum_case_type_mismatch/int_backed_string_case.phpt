===description===
P6(a): An int-backed enum case with a string value must be flagged.
The correctly-typed case (High = 1) must NOT produce an error.
===file===
<?php
enum Priority: int {
    case Low = 'low';
//             ^^^^^ BackedEnumCaseTypeMismatch: Enum case type string does not match enum backing type int
    case High = 1;
}
