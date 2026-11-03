===description===
A stub's `#[Pure]` is an IDE hint about the builtin, so a user override of it isn't checked as pure; a call to the non-overridden builtin method keeps its purity.
===file===
<?php
final class Rot extends \Transliterator {
    private ?string $cookie = null;

    public function transliterate(string $string, int $start = 0, int $end = -1): string|false {
        $this->cookie ??= uniqid();
        return $this->cookie . $string;
    }
}

/** @pure */
function errorCode(Rot $r): int {
    return $r->getErrorCode();
}
