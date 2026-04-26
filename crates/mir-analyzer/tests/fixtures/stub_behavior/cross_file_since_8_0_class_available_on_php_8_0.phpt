===config===
php_version=8.0
===file:Cache.php===
<?php
function make_weak_cache(): void {
    new WeakMap();
}
===file:App.php===
<?php
make_weak_cache();
===expect===
