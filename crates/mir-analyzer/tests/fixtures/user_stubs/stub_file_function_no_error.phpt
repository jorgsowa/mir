===description===
stub file function no error
===config===
stub_file=stubs/helpers.php
suppress=UnusedVariable
===file:stubs/helpers.php===
<?php
function my_helper(string $s): string { return $s; }
===file:App.php===
<?php
$result = my_helper('hello');
===expect===
