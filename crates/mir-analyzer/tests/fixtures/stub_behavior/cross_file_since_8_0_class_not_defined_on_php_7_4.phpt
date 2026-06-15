===description===
cross file since 8 0 class not defined on php 7 4
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
Cache.php: UndefinedClass@3:8-3:15: Class WeakMap does not exist
