===description===
method_exists() guard suppresses UndefinedMethod for a 'Foo::method' string callable
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
        register_shutdown('Other::gateStatic');
    }
}
===expect===
