===description===
Outside the true branch of `&&` a method's out param is still possibly
undefined, because the call may have been short-circuited.
===file===
<?php
final class Parser {
    public function parse(string $in, ?int &$out): bool {
        $out = (int) $in;
        return true;
    }
}

function f(string $in, bool $b, Parser $p): int {
    if ($b && $p->parse($in, $out)) {
        return 1;
    }
    return $out ?? 0;
}

function g(string $in, bool $b, Parser $p): ?int {
    if ($b && $p->parse($in, $out)) {
        return 1;
    }
    return $out;
//         ^^^^ PossiblyUndefinedVariable: Variable $out might not be defined
}
