===description===
cross file since 8 0 class not defined on php 7 4
===config===
php_version=7.4
===file:Cache.php===
<?php
function make_weak_cache(): void {
    new WeakMap();
//      ^^^^^^^ UndefinedClass: Class WeakMap does not exist
}
===file:App.php===
<?php
make_weak_cache();
===expect===
