===description===
FP-P11(b): PREG_UNMATCHED_AS_NULL combined with PREG_OFFSET_CAPTURE (256|512=768).
Real PHP reports unmatched groups as [null, -1] in this mode, so the offset-capture
shape's text slot (index 0) must admit null too, not just the plain string leaf.
Uses the literal flags value 768 rather than the `|` expression: mir does not
constant-fold a bitwise-OR of two literal ints into a literal int (a separate,
pre-existing gap unrelated to preg_match), so `PREG_UNMATCHED_AS_NULL |
PREG_OFFSET_CAPTURE` still resolves to a generic int and would not exercise the
combined-flags branch under test here.
===config===
suppress=UnusedVariable,UnusedFunction,MixedArgument
php_version=8.2
===file===
<?php

function parseNumber(string $value): void {
    preg_match('/(?P<integral>\d+)(\.(?P<fraction>\d+))?/', $value, $matches, 768);

    if ($matches['fraction'][0] === null) {
        echo "no fraction\n";
    }
}
===expect===
