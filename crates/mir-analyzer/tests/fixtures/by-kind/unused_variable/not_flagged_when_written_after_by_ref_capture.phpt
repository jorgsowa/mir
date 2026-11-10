===description===
A variable a closure captured by reference is read through that reference, so a write after the capture is not unused; a write nothing captures still is.
===file===
<?php
function installHandler(): void {
    $previous = set_error_handler(static function (int $level, string $message) use (&$previous): bool {
        return $previous ? (bool) $previous($level, $message) : false;
    });
}

function reportAtShutdown(): void {
    $status = 'starting';
    register_shutdown_function(static function () use (&$status): void {
        echo $status;
    });
    $status = 'finished';
}

function definedAfterCapture(): void {
    register_shutdown_function(static function () use (&$summary): void {
        echo is_string($summary) ? $summary : 'none';
    });
    $summary = 'done';
}

function neverCaptured(): void {
    $status = 'starting';
    echo $status;
    $status = 'finished';
//  ^^^^^^^ UnusedVariable: Variable $status is never read
}
