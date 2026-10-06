===description===
P6(a): Each mismatched case in a string-backed enum emits a separate issue.
===file===
<?php
enum Color: string {
    case Red = 1;
//             ^ BackedEnumCaseTypeMismatch: Backed enum case Color::Red has value of type 1, but backing type is string
    case Green = 2;
//               ^ BackedEnumCaseTypeMismatch: Backed enum case Color::Green has value of type 2, but backing type is string
    case Blue = 'blue';
}
