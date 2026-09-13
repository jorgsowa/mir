===description===
`method_exists()` suppresses undefined methods for guarded class-string callables.
===config===
suppress=MissingReturnType
===file===
<?php
class Other {}

function register_shutdown(callable $cb): void {
    $cb();
}

function free(): void {
    if (method_exists(Other::class, 'gateStatic')) {
        register_shutdown([Other::class, 'gateStatic']);
    }
}
===expect===
