===config===
php_version=7.4
===file:StringHelper.php===
<?php
function check_contains(string $text, string $needle): void {
    str_contains($text, $needle);
}
===file:App.php===
<?php
check_contains('hello world', 'world');
===expect===
StringHelper.php: UndefinedFunction: Function str_contains() is not defined
