===description===
calling a function defined only in a stub file without stub_file config emits UndefinedFunction
===config===
suppress=MixedAssignment,UnusedFunction,UnusedVariable
===file:App.php===
<?php
function test(): void {
    $keys = array_key_list(['x' => 1, 'y' => 2]);
    $_ = $keys;
}
===expect===
App.php: UndefinedFunction@3:13-3:49: Function array_key_list() is not defined
