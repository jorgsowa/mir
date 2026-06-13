===description===
tainted payload reaching unserialize is reported (object injection)
===config===
suppress=MixedArgument,MixedArrayAccess,MixedAssignment,UnusedVariable
===file===
<?php
function test(): void {
    $obj = unserialize($_COOKIE['session']);
}
===expect===
TaintedInput@3:12-3:44: Tainted input reaching sink 'unserialize'
