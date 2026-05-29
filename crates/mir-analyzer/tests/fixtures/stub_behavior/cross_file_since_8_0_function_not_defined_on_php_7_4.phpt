===description===
cross file since 8 0 function not defined on php 7 4
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
StringHelper.php: UndefinedFunction@3:5-3:33: Function str_contains() is not defined
