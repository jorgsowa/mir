===description===
P27 regression guard, param-side: a genuine docblock/native-hint conflict on a
shadowed-builtin-name parameter must still be flagged — the docblock claims `Other`
(a real, different class) while the native hint says `Generator` (the local shadow).
===config===
suppress=UnusedParam
===file===
<?php

namespace App;

final class Generator
{
}

final class Other
{
}

/**
 * @param Other $g
 */
function useIt(Generator $g): void
{
}
===expect===
MismatchingDocblockParamType@16:25-16:27: Docblock type 'App\Other' for $g does not match inferred 'App\Generator'
