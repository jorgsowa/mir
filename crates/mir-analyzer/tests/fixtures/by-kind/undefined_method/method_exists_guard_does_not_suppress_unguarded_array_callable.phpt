===description===
Unguarded [Foo::class, 'method'] array callable still reports UndefinedMethod
===config===
suppress=MissingReturnType
===file===
<?php
class Other {}

function register_shutdown(callable $cb): void {
    $cb();
}

function free(): void {
    register_shutdown([Other::class, 'gateStatic']);
}
===expect===
UndefinedMethod@9:22-9:50: Method Other::gateStatic() does not exist
