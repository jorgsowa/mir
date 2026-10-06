===description===
calling a function defined only in a stub file without stub_file config emits UndefinedFunction
===config===
<mir>
  <issueHandlers>
    <MixedAssignment errorLevel="suppress"/>
    <UnusedFunction errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file:App.php===
<?php
function test(): void {
    $keys = array_key_list(['x' => 1, 'y' => 2]);
//          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UndefinedFunction: Function array_key_list() is not defined
    $_ = $keys;
}
