===config===
stub_file=stubs/polyfill.php
===file:stubs/polyfill.php===
<?php
function strlen(string $string, string $encoding = 'UTF-8'): int { return 0; }
===file:App.php===
<?php
strlen('hello', 'ASCII');
===expect===
