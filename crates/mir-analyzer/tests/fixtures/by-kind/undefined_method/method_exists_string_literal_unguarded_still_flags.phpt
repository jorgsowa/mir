===description===
Without a method_exists() guard, a 'Foo::method' string callable whose
target method does not exist is checked normally and UndefinedMethod is
reported.
===config===
suppress=MissingReturnType
===file===
<?php
class Other {}

function register_shutdown(callable $cb): void {
    $cb();
}

function free(): void {
    register_shutdown('Other::gateStatic');
}
===expect===
UndefinedMethod@9:22-9:41: Method Other::gateStatic() does not exist
