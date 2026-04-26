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
Async.php: UndefinedClass: Class Fiber does not exist
