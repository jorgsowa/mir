===description===
cross file since 8 1 class not defined on php 8 0
===config===
php_version=8.0
===file:Async.php===
<?php
function make_fiber(callable $fn): void {
    new Fiber($fn);
}
===file:App.php===
<?php
make_fiber(function (): void {});
===expect===
Async.php: UndefinedClass@3:8: Class Fiber does not exist
