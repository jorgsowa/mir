===description===
tainted payload reaching unserialize is reported (object injection)
===config===
suppress=MixedArgument,MixedArrayAccess,MixedAssignment,UnusedVariable
===file===
<?php
function test(): void {
    $obj = unserialize($_COOKIE['session']);
//         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedInput: Tainted input reaching sink 'unserialize'
}
===expect===
