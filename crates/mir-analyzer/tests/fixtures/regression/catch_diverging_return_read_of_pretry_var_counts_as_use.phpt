===description===
Same underlying bug as catch_diverging_rethrow_read_counts_as_use, but the
diverging catch clause returns instead of throwing, and the variable it
reads was assigned BEFORE the try (not a parameter). Confirms the fix isn't
narrowly tied to "throw" as the divergence mechanism, and that a diverging
catch's read of a pre-try variable also suppresses that earlier write from
being reported dead.
===file===
<?php
function openConnection(): object {
    return new \stdClass();
}

function doWork(): void {}

function logError(object $conn): void {
    echo get_class($conn);
}

function g(): void {
    $conn = openConnection();
    try {
        doWork();
    } catch (\Throwable $e) {
        logError($conn);
        return;
    }
}
===expect===
