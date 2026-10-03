===description===
The File sink check inspected the raw physical argument position, not the
resolved parameter -- a PHP 8 named-argument call that reorders arguments
(data before filename) moved the tainted path off index 0, defeating the
positional check even though it's still the same $filename parameter.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function test(): void {
    $path = $_GET['name'];
    file_put_contents(data: 'safe-constant', filename: $path);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedInput: Tainted input reaching sink 'file'
}
===expect===
