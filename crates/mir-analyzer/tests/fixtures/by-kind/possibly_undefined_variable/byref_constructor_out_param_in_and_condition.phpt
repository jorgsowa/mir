===description===
A constructor's by-ref param defines the out variable in the true branch of `&&`.
===file===
<?php
final class Reader {
    public function __construct(public string $path, ?string &$error) {
        $error = $path === '' ? 'empty' : null;
    }
}

function open(string $path, bool $b): ?string {
    if ($b && ($reader = new Reader($path, $error))) {
        /** @mir-check $error is string|null */
        return $error ?? $reader->path;
    }
    return null;
}
