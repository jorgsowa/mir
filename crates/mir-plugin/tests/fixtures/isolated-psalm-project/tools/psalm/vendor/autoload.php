<?php

// Autoloader of a Psalm install isolated from the project's own vendor dir.
spl_autoload_register(static function (string $class): void {
    $map = [
        'Psalm\\' => __DIR__ . '/../../../../fake-psalm-project/fake-psalm/',
        'PhpParser\\' => __DIR__ . '/../../../../fake-psalm-project/fake-php-parser/',
    ];
    foreach ($map as $prefix => $dir) {
        if (str_starts_with($class, $prefix)) {
            $file = $dir . str_replace('\\', '/', substr($class, strlen($prefix))) . '.php';
            if (is_file($file)) {
                require $file;
            }
            return;
        }
    }
});
