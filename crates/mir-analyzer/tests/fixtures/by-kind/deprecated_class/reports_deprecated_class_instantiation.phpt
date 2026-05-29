===description===
reports deprecated class instantiation
===file===
<?php
/** @deprecated use NewClass instead */
class OldClass {}

function test(): void {
    $obj = new OldClass();
}
===expect===
UnusedVariable@6:5-6:9: Variable $obj is never read
DeprecatedClass@6:16-6:24: Class OldClass is deprecated: use NewClass instead
