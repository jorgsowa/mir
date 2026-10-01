<?php

// Project autoloader that knows the plugin but not Psalm.
spl_autoload_register(static function (string $class): void {
    if (str_starts_with($class, 'TestPlugin\\')) {
        $file = __DIR__ . '/../../fake-psalm-project/plugin/'
            . str_replace('\\', '/', substr($class, strlen('TestPlugin\\'))) . '.php';
        if (is_file($file)) {
            require $file;
        }
    }
});
