===description===
FP-A: class_exists() / interface_exists() guards must suppress UndefinedClass
for optional dependencies: both the direct if-block form and the negative
early-return form.
===config===
php_version=8.2
===file===
<?php

// Direct guard form
if (class_exists(\Redis::class)) {
    $redis = new \Redis();
    $redis->connect('127.0.0.1');
}

// String-literal form
if (class_exists('Memcached')) {
    $_ = new Memcached();
}

// interface_exists form
if (interface_exists(\Countable::class)) {
    $_ = new class implements \Countable {
        public function count(): int { return 0; }
    };
}
===expect===

