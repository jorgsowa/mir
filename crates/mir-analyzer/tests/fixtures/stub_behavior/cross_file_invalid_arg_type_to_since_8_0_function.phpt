===description===
cross file invalid arg type to since 8 0 function
===config===
php_version=8.0
===file:StringHelper.php===
<?php
function test_wrong_type(int $n): void {
    str_contains($n, 'needle');
//               ^^ ArgumentTypeCoercion: Argument $haystack of str_contains() expects 'string', got 'int' — coercion may fail at runtime
}
===file:App.php===
<?php
test_wrong_type(42);
===expect===
