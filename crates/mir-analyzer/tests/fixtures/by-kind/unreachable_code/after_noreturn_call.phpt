===description===
Code after a call to a `@return noreturn` function or method is unreachable.
===file===
<?php
/** @return noreturn */
function fail(string $message) {
    throw new \RuntimeException($message);
}

final class Guard {
    /** @return noreturn */
    public function abort() {
        exit(1);
    }
}

function afterFunction(): int {
    fail('stop');
    return 1;
//  ^^^^^^^^^ UnreachableCode: Unreachable code detected
}

function afterMethod(Guard $guard): int {
    $guard->abort();
    return 1;
//  ^^^^^^^^^ UnreachableCode: Unreachable code detected
}

function reachable(bool $bad): int {
    if ($bad) {
        fail('stop');
    }
    return 1;
}
