===description===
The File sink check inspected the raw physical argument position, not the
resolved parameter -- a PHP 8 named-argument call that reorders arguments
(data before filename) moved the tainted path off index 0, defeating the
positional check even though it's still the same $filename parameter.
===config===
suppress=MixedArgument,MixedArrayAccess,MixedAssignment,UnusedVariable
===file===
<?php
function test(): void {
    $path = $_GET['name'];
    file_put_contents(data: 'safe-constant', filename: $path);
}
===expect===
TaintedInput@4:4-4:61: Tainted input reaching sink 'file'
