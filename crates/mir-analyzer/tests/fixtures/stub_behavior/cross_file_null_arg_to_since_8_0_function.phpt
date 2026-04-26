===config===
php_version=8.0
===file:StringHelper.php===
<?php
function test_null(string $needle): void {
    str_contains(null, $needle);
}
===file:App.php===
<?php
test_null('hello');
===expect===
StringHelper.php: NullArgument: Argument $haystack of str_contains() cannot be null
