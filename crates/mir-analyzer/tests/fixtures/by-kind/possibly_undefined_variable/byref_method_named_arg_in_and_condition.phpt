===description===
A named argument binds to the method's by-ref param, so it defines the out
variable in the true branch of `&&`.
===file===
<?php
final class Parser {
    public function parse(string $in, ?int &$out = null, bool $strict = false): bool {
        $out = $strict ? (int) $in : null;
        return true;
    }
}

function f(string $in, bool $b, Parser $p): int {
    if ($b && $p->parse(strict: true, out: $out, in: $in)) {
        return $out ?? 0;
    }
    return 0;
}
