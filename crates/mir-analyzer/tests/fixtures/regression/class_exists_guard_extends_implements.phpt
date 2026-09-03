===description===
FP-A: optional-dependency pattern — class_exists() throw guard before a class
declaration using extends/implements should suppress UndefinedClass. Without
the fix, the analyzer unconditionally checks the parent/interface name regardless
of the preceding guard.
===config===
suppress=UnusedVariable,UnusedParam
php_version=8.2
===file===
<?php

// Pattern 1: extends with throw guard
if (!class_exists(\NonExistent\OptionalParent::class)) {
    throw new \RuntimeException('Optional dependency not installed.');
}

class ConcreteChild extends \NonExistent\OptionalParent {}

// Pattern 2: implements with string-literal guard
if (!interface_exists('NonExistent\OptionalInterface')) {
    throw new \LogicException('Optional interface not available.');
}

class ConcreteImpl implements \NonExistent\OptionalInterface {}

// Pattern 3: return guard (early exit from bootstrap file)
if (!class_exists(\NonExistent\AnotherOptional::class)) {
    return;
}

class UsesAnotherOptional extends \NonExistent\AnotherOptional {}
===expect===
