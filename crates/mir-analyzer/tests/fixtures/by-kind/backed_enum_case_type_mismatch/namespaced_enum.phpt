===description===
P6(a): BackedEnumCaseTypeMismatch includes the fully-qualified enum name in namespaced code.
===file===
<?php
namespace App\Enums;

enum Severity: int {
    case Low = 'low';
//             ^^^^^ BackedEnumCaseTypeMismatch: Enum case type string does not match enum backing type int
    case High = 3;
}
