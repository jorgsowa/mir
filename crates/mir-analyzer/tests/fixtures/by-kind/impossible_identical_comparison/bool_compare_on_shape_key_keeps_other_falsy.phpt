===description===
Comparing a shape key to a bool rules out only that bool, not every falsy value, unlike a bare truthiness check.
===file===
<?php
/** @param array{v: string|false|null} $m */
function false_then_null(array $m): void {
    if (false === $m['v']) {
        echo 1;
    } elseif (null !== $m['v']) {
        echo 2;
    }
}

/** @param array{v: int|false|null} $m */
function after_return(array $m): void {
    if ($m['v'] === false) {
        return;
    }
    if ($m['v'] !== null) {
        echo 2;
    }
}

/** @param array{v: string|bool|null} $m */
function not_true_is_not_falsy(array $m): void {
    if (true === $m['v']) {
        return;
    }
    if ($m['v'] !== 'x') {
        echo 2;
    }
}

/** @param array{v: string|null} $m */
function bare_check_still_narrows(array $m): void {
    if ($m['v']) {
        echo strlen($m['v']);
    }
}
