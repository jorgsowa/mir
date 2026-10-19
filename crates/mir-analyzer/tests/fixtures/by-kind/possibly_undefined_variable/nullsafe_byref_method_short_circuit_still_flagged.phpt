===description===
A nullsafe call on a null receiver skips its arguments, so its out param
stays possibly undefined even when the condition holds.
===file===
<?php
final class Parser {
    public function parse(string $in, ?int &$out): bool {
        $out = (int) $in;
        return true;
    }
}

function f(string $in, bool $b, ?Parser $p): ?int {
    if ($b && $p?->parse($in, $out) === null) {
        return $out;
//             ^^^^ PossiblyUndefinedVariable: Variable $out might not be defined
    }
    return null;
}
