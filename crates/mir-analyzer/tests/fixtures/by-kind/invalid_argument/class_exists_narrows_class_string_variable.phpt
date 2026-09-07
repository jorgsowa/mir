===description===
FP-A(b): class_exists($var) guard must narrow $var from string to class-string so
that passing it to a class-string-typed parameter does not emit InvalidArgument.
===config===
suppress=UnusedVariable
php_version=8.2
===file===
<?php

/** @param class-string $cls */
function instantiate(string $cls): object {
    return new $cls();
}

// Guard form: true branch
function createGuarded(string $className): object {
    if (class_exists($className)) {
        return instantiate($className);
    }
    throw new \RuntimeException("Class $className not found");
}

// Early-exit form: after negative guard $className is class-string
function createWithEarlyExit(string $className): object {
    if (!class_exists($className)) {
        throw new \RuntimeException("Class $className not found");
    }
    return instantiate($className);
}

// interface_exists narrows too
/** @param class-string $iface */
function describeInterface(string $iface): string {
    return "interface: $iface";
}

function describeIfExists(string $iface): string {
    if (interface_exists($iface)) {
        return describeInterface($iface);
    }
    return 'not found';
}
===expect===
