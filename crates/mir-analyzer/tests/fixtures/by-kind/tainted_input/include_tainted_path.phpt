===description===
`include`/`require`/`include_once`/`require_once` with a tainted path was
entirely unchecked — like `eval()`, this is its own AST node (not a
FunctionCall), so the taint-sink dispatch keyed on function names never saw
it. A tainted path here is local/remote file inclusion.
===config===
suppress=MixedArgument,MixedArrayAccess,MixedAssignment,UnusedVariable
===file===
<?php
function test(): void {
    $page = $_GET['page'];
    include $page . '.php';
//  ^^^^^^^^^^^^^^^^^^^^^^ TaintedInput: Tainted input reaching sink 'include'
}

function testRequire(): void {
    require $_GET['page'];
//  ^^^^^^^^^^^^^^^^^^^^^ TaintedInput: Tainted input reaching sink 'include'
}

function testSafe(): void {
    include 'header.php';
}
===expect===
