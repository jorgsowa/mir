===description===
M21: ReflectionClass::hasMethod()/ReflectionParameter::isDefaultValueAvailable()
guard their twin throwing call (getMethod()/getDefaultValue()) on the same
receiver — sibling of the method_exists()/property_exists() free-function
guards, but for Reflection's own instance API. An unguarded call on a
different receiver, or the same receiver without the guard, still flags.
===config===
suppress=UnusedParam
===file===
<?php
function guardedMethod(string $c, string $n): void {
    $class = new \ReflectionClass($c);
    if ($class->hasMethod($n)) {
        $class->getMethod($n);
    }
}

function unguardedMethod(string $c, string $n): void {
    $class = new \ReflectionClass($c);
    $class->getMethod($n);
}

function guardedDefault(\ReflectionParameter $p): void {
    if ($p->isDefaultValueAvailable()) {
        $p->getDefaultValue();
    }
}

function unguardedDefault(\ReflectionParameter $p): void {
    $p->getDefaultValue();
}
===expect===
MissingThrowsDocblock@11:4-11:25: Exception ReflectionException is thrown but not declared in @throws
MissingThrowsDocblock@21:4-21:25: Exception ReflectionException is thrown but not declared in @throws
