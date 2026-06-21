===description===
`false === $x` and `false === ($x = expr)` guards with a throw should narrow $x to
exclude false in the continuation. Previously only `$x === false` was handled.
The assignment-in-condition form `false === ($x = expr)` is common in intl/normalizer
patterns (UnicodeString-style).
===config===
[analysis]
php_version = "8.1"
===file===
<?php
declare(strict_types=1);

class Normalizer {
    private string $value = '';

    // `false === $x` (literal on left, separate assignment)
    public function setFromNormalized1(string $input): void {
        $out = normalizer_normalize($input);
        if (false === $out) {
            throw new \InvalidArgumentException('invalid UTF-8');
        }
        $this->value = $out;
    }

    // `false === ($x = expr)` — assignment in condition, literal on left
    public function setFromNormalized2(string $input): void {
        if (false === $out = normalizer_normalize($input)) {
            throw new \InvalidArgumentException('invalid UTF-8');
        }
        $this->value = $out;
    }

    // `true === ($x = expr)` — same pattern with true
    public function setIfTrue(bool $flag): void {
        if (true === $flag) {
            $this->value = 'enabled';
        }
    }
}
===expect===
No errors
