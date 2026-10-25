===description===
P6(a): A string-backed enum case with an integer value must be flagged.
The correctly-typed case (Inactive = 'inactive') must NOT produce an error.
===file===
<?php
enum Status: string {
    case Active = 1;
//                ^ BackedEnumCaseTypeMismatch: Enum case type int does not match enum backing type string
    case Inactive = 'inactive';
}
