===description===
P6(a): Each mismatched case in a string-backed enum emits a separate issue.
===file===
<?php
enum Color: string {
    case Red = 1;
//             ^ BackedEnumCaseTypeMismatch: Enum case type int does not match enum backing type string
    case Green = 2;
//               ^ BackedEnumCaseTypeMismatch: Enum case type int does not match enum backing type string
    case Blue = 'blue';
}
