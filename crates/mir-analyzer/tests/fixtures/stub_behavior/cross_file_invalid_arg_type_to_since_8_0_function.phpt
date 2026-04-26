===config===
php_version=8.0
===file:StringHelper.php===
<?php
function test_wrong_type(int $n): void {
    str_contains($n, 'needle');
}
===file:App.php===
<?php
test_wrong_type(42);
===expect===
StringHelper.php: InvalidArgument: Argument $haystack of str_contains() expects 'string', got 'int'
