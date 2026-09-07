===description===
A class-string<Interface>-typed variable used as a :: receiver holds the
class-string of whatever concrete implementing class it was assigned — valid
PHP, must analyze clean.
===config===
suppress=UnusedParam
===file===
<?php

interface Provider {
    public static function getDefinitions(): array;
}

/** @param class-string<Provider> $provider */
function run(string $provider): void {
    $provider::getDefinitions();
}
===expect===
