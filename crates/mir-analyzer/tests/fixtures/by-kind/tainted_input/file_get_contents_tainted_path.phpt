===description===
tainted path reaching file_get_contents is reported (path traversal / SSRF)
===config===
suppress=MixedArgument,MixedArrayAccess,MixedAssignment,UnusedVariable
===file===
<?php
function test(): void {
    $path = $_GET['path'];
    $data = file_get_contents($path);
}
===expect===
TaintedInput@4:12-4:36: Tainted input reaching sink 'file'
