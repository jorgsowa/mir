===description===
FP-A: extension_loaded() guard suppresses UndefinedClass for optional PHP extensions.
Both the direct if-block form and the negative early-exit form must work.
Classes used inside the guarded block are assumed to come from the extension.
===config===
php_version=8.2
suppress=UnusedVariable,UnusedParam
===file===
<?php

// Direct guard form — new and static call both suppressed
if (extension_loaded('my_custom_ext')) {
    $obj = new \MyCustomExtClass();
    \MyCustomExtClass::configure();
}

// Negative early-exit form (throw)
function requireCustomExt(): void {
    if (!extension_loaded('my_ext')) {
        throw new \RuntimeException('my_ext required');
    }
    $obj = new \MyExtClass();
}

// Negative early-exit form (return)
function setupIfAvailable(): void {
    if (!extension_loaded('another_ext')) {
        return;
    }
    $obj = new \AnotherExtClass();
}

// Static method call inside guard
function callStatic(): void {
    if (!extension_loaded('my_static_ext')) {
        return;
    }
    \MyStaticExtClass::doWork();
}
===expect===
