===config===
stub_file=stubs/polyfill.php
===file:stubs/polyfill.php===
<?php
function strlen(string $string, string $encoding): int { return 0; }
===file:App.php===
<?php
strlen('hello');
===expect===
App.php: TooFewArguments: Too few arguments for strlen(): expected 2, got 1
