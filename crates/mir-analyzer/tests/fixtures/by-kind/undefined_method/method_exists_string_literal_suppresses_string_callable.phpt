===description===
method_exists() with a string-literal class name registers the same guard
key as the `::class` form, suppressing UndefinedMethod for a
'Foo::method' string callable inside the guarded branch.
===config===
suppress=MissingReturnType
===file===
<?php
class Other {}

function register_shutdown(callable $cb): void {
    $cb();
}

function free(): void {
    if (method_exists('Other', 'gateStatic')) {
        register_shutdown('Other::gateStatic');
    }
}
===expect===
