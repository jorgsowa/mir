===description===
At do/while body entry the type is initial ∪ narrow(back-edge); the condition must not narrow the initial state.
===file===
<?php
/** @return non-empty-string|null */
function fetch_next(): ?string { return rand(0, 1) ? 'x' : null; }

function not_null_check(): void {
    $token = null;
    do {
        /** @mir-check $token is non-empty-string|null */
        if ($token !== null) { echo $token; }
        $token = fetch_next();
    } while (is_string($token) && $token !== '');
}

function null_check(): void {
    $token = null;
    do {
        if ($token === null) { echo 'first'; }
        $token = fetch_next();
    } while ($token !== null);
}

function nested_in_loop(): void {
    foreach ([1, 2] as $_) {
        $token = null;
        do {
            if ($token !== null) { echo $token; }
            $token = fetch_next();
        } while (is_string($token));
    }
}

function narrowing_still_applies_on_back_edge(): void {
    do {
        $token = fetch_next();
    } while ($token !== null);
    /** @mir-check $token is null */
    echo $token;
}

function condition_not_applied_to_first_iteration(?string $s): void {
    do {
        /** @mir-check $s is string|null */
        echo '';
        $s = rand(0, 1) ? 'a' : null;
    } while ($s !== null);
}
===expect===
