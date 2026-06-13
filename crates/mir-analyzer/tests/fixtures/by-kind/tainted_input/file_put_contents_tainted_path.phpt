===description===
a tainted path passed to file_put_contents is reported even when the data is safe
===config===
suppress=MixedArgument,MixedArrayAccess,MixedAssignment,UnusedVariable
===file===
<?php
function test(): void {
    $path = $_GET['name'];
    file_put_contents($path, 'safe-constant');
}
===expect===
TaintedInput@4:5-4:46: Tainted input reaching sink 'file'
