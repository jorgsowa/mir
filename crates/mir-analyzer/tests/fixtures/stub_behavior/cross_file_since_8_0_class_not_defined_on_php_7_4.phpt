===config===
php_version=7.4
===file:Cache.php===
<?php
function make_weak_cache(): void {
    new WeakMap();
}
===file:App.php===
<?php
make_weak_cache();
===expect===
Cache.php: UndefinedClass: Class WeakMap does not exist
