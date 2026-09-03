===description===
An undefined `use (&$var)` by-ref capture was unconditionally seeded as a
callable of unknown arity, on the assumption that the dominant case is a
self-referential closure (`$f = function () use (&$f) {...}`). The equally
common "by-ref out-param" idiom — a callback that reports a value back
through a captured-but-not-yet-defined variable, with no enclosing
assignment at all — got the same non-nullable callable type, so a later
guard on that variable misfired RedundantCondition (always true) and
throwing it misfired InvalidThrow ("callable does not extend Throwable").
Modeled on doctrine/instantiator's use of `set_error_handler()`.
===file===
<?php
function run(): void {
    set_error_handler(function (int $errno, string $errstr) use (&$error): bool {
        $error = new \RuntimeException($errstr);
        return true;
    });
    if ($error) {
        throw $error;
    }
}
===expect===
