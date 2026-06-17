===description===
Foo::class is a PHP compile-time string constant — the class need not be loaded or
defined. UndefinedClass must never fire for ::class expressions regardless of context:
inside class_exists(), on the false branch of a ternary, or as a plain argument.
===config===
suppress=UnusedVariable,MissingParamType,UnusedParam
===file===
<?php
// Direct class_exists() argument
$e1 = class_exists(\Optional\Package::class);

// Ternary: true branch and false branch both use ::class of undefined classes.
$cls = class_exists(\Optional\NewVersion::class)
    ? \Optional\NewVersion::class
    : \Optional\LegacyVersion::class;

// ::class used as a container make() argument — common Laravel pattern.
/** @param object $container */
function make_it($container): void {
    $container->make(\Optional\SomeService::class);
}
===expect===
