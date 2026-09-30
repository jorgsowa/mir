===description===
`move_uploaded_file($from, $to)` was entirely missing from the File sink
list — a tainted destination path (arbitrary file write / path traversal)
went unchecked. Unlike every other File sink, the dangerous argument is the
SECOND one ($to), not the first, so the positional fallback needed its own
override alongside the named-argument resolution.
===config===
suppress=MixedArgument,MixedArrayAccess,MixedAssignment,UnusedVariable
===file===
<?php
function test(): void {
    $dest = $_GET['dest'];
    move_uploaded_file('/tmp/upload_tmp', $dest);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedInput: Tainted input reaching sink 'file'
}

function testNamedArgs(): void {
    move_uploaded_file(from: '/tmp/upload_tmp', to: $_GET['dest']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedInput: Tainted input reaching sink 'file'
}

function testSafe(): void {
    move_uploaded_file($_GET['tmp'], '/var/uploads/fixed-name.bin');
}
===expect===
