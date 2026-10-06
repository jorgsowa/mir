===description===
P6(a): Each mismatched case in a string-backed enum emits a separate issue.
===file===
<?php
enum Color: string {
    case Red = 1;
    case Green = 2;
    case Blue = 'blue';
}
===expect===
BackedEnumCaseTypeMismatch@3:15-3:16: Backed enum case Color::Red has value of type 1, but backing type is string
BackedEnumCaseTypeMismatch@4:17-4:18: Backed enum case Color::Green has value of type 2, but backing type is string
