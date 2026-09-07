===description===
P27, param-side: same as the return-type case, but for `@param`/native param hint
agreement. `MismatchingDocblockParamType` must not fire when both the docblock and
the native param hint bare-name a same-namespace class shadowing a builtin
iterator-family name — the two are read the same way once the docblock side's bare
builtin-leniency is reconciled against a confirmed local shadow.
===config===
suppress=UnusedParam
===file===
<?php

namespace App;

final class Generator
{
}

/**
 * @param Generator $g
 */
function useIt(Generator $g): void
{
}
===expect===
