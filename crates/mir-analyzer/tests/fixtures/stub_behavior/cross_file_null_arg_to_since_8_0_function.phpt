===description===
cross file null arg to since 8 0 function
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
StringHelper.php: NullArgument@3:17-3:21: Argument $haystack of str_contains() cannot be null
