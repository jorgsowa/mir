===description===
Same root cause as byref_use_out_param_not_seeded_as_callable, but the
closure is first assigned to a variable before being passed on — proving
the fix keys off the CAPTURED variable's own name matching the assignment
target, not merely "a closure literal appears somewhere near an
assignment". Here the assignment target (`$handler`) and the by-ref
capture (`$error`) are different names, so this is unambiguously the
out-param idiom, never the self-referential one.
===file===
<?php
function run(): void {
    $handler = function (int $errno, string $errstr) use (&$error): bool {
        $error = new \RuntimeException($errstr);
        return true;
    };
    set_error_handler($handler);
    if ($error) {
        throw $error;
    }
}
===expect===
