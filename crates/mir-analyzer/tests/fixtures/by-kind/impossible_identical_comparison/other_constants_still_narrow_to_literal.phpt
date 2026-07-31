===description===
Negative control for the I7 environment-constant-widening fix: an ordinary
constant NOT in the small environment-dependent allowlist must still be
narrowed to its literal value — the widening must not generalize to every
global constant.
===config===
suppress=UnusedFunction
===file===
<?php

define('MY_CONST', 'foo');

function f(): bool {
    return MY_CONST === 'bar';
}
===expect===
ImpossibleIdenticalComparison@6:11-6:29: '===' between '"foo"' and '"bar"' is always false — these types can never be identical
