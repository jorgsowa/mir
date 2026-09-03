===description===
P27 regression guard: the reconciliation added for the shadowed-builtin-name case
must only ever narrow a bare builtin docblock name to a *confirmed* local shadow —
a genuine, unrelated docblock/native-hint conflict on the very same shadowed class
must still be flagged. Here the docblock claims `Other` (a real, different class)
while the native hint says `Generator` (the local shadow) — no reconciliation
applies since `Other` isn't a builtin-leniency name, so the real contradiction must
still surface.
===file===
<?php

namespace App;

final class Generator
{
    public function build(): string { return 'built'; }
}

final class Other
{
}

/**
 * @return Other
 */
function make(): Generator
{
    return new Generator();
}
===expect===
MismatchingDocblockReturnType@17:9-17:13: Docblock return type 'App\Other' does not match inferred 'App\Generator'
InvalidReturnType@19:4-19:27: Return type 'App\Generator' is not compatible with declared 'App\Other'
