===description===
reports deprecated class instantiation
===file===
<?php
/** @deprecated use NewClass instead */
class OldClass {}

function test(): void {
    $obj = new OldClass();
//  ^^^^ UnusedVariable: Variable $obj is never read
//             ^^^^^^^^ DeprecatedClass: Class OldClass is deprecated: use NewClass instead
}
