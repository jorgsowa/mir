===description===
FN: a `match ($x) { 1, 2 => ... }` arm never actually narrowed `$x` to `1|2`
inside its body, even though `analyze_match` computes exactly that narrowed
type and calls `arm_ctx.set_var` with it. The root cause was in
`Type::intersect_with`: intersecting `int` (the wider, self side) with `1|2`
(the narrower, other side) kept `int` — the WIDER atomic — instead of the
narrower `1|2`, because the loop always kept `self`'s own atomic on any
match rather than whichever side was more specific. `@mir-check` on `$r`
(built via an identity call inside the arm, since match arms are
expressions and can't carry their own doc comment) surfaces the arm-local
narrowed type of `$x`.
===config===
suppress=UnusedVariable
===file===
<?php
/**
 * @template T
 * @param T $y
 * @return T
 */
function identity($y) {
    return $y;
}

function f(int $x): void {
    $r = match ($x) {
        1, 2 => identity($x),
        default => 0,
    };
    /** @mir-check $r is 1|2|0 */
    echo "ok";
}
===expect===
