===description===
PHP 8.0 built-in functions (str_contains, str_starts_with, str_ends_with) must be
reported as undefined on a PHP 7.4 session. Regression guard for the pull-path
version-filtering bug where collect_file_definitions always used the db default
(8.2) instead of the configured target version.
===config===
php_version=7.4
suppress=MixedReturnStatement
===file:App.php===
<?php
function check(string $s): bool {
    return str_contains($s, 'x');
}
===file:Other.php===
<?php
require_once 'App.php';
check('hello');
===expect===
App.php: UndefinedFunction@3:12-3:33: Function str_contains() is not defined
