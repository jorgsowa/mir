===description===
P6(a): A string-backed enum case with an integer value must be flagged.
The correctly-typed case (Inactive = 'inactive') must NOT produce an error.
===file===
<?php
enum Status: string {
    case Active = 1;
    case Inactive = 'inactive';
}
===expect===
BackedEnumCaseTypeMismatch@3:0-3:0: Backed enum case Status::Active has value of type 1, but backing type is string
