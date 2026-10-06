===description===
P6(a): BackedEnumCaseTypeMismatch includes the fully-qualified enum name in namespaced code.
===file===
<?php
namespace App\Enums;

enum Severity: int {
    case Low = 'low';
//             ^^^^^ BackedEnumCaseTypeMismatch: Backed enum case App\Enums\Severity::Low has value of type "low", but backing type is int
    case High = 3;
}
===expect===
