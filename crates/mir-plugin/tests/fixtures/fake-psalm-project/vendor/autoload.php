<?php

// Hand-rolled autoloader standing in for composer's, mapping the minimal
// slice of Psalm's (and PhpParser's) API surface the mir plugin host touches,
// plus the fixture plugin itself.
spl_autoload_register(static function (string $class): void {
    $map = [
        'Psalm\\' => __DIR__ . '/../fake-psalm/',
        'PhpParser\\' => __DIR__ . '/../fake-php-parser/',
        'TestPlugin\\' => __DIR__ . '/../plugin/',
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
