===description===
a constant path is never a taint sink
===config===
suppress=MixedArgument,MixedAssignment,UnusedVariable
===file===
<?php
function test(): void {
    $data = file_get_contents('/etc/hostname');
    $obj = unserialize('a:0:{}');
}
===expect===
