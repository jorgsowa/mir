===description===
stub file function type checked
===config===
stub_file=stubs/helpers.php
===file:stubs/helpers.php===
<?php
function my_helper(string $s): string { return $s; }
===file:App.php===
<?php
function test(): void { my_helper(42); }
===expect===
App.php: InvalidArgument@2:34: Argument $s of my_helper() expects 'string', got '42'
