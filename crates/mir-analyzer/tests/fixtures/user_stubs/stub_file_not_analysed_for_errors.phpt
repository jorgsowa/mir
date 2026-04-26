===config===
stub_file=stubs/helpers.php
===file:stubs/helpers.php===
<?php
// Stub files are not analysed — errors inside them must not be reported.
function my_helper(string $s): void { undeclared_call(); }
===file:App.php===
<?php
function test(): void { my_helper('hello'); }
===expect===
