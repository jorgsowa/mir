===description===
P28: a top-level function's `@param` docblock naming a same-namespace class
that shadows a builtin (`Generator`) must resolve to that local class for
STORAGE, not just for the P27 comparison — the stored, canonical param type
must carry the qualified FQCN into the body's own flow tracking.
===file===
<?php

namespace App;

final class Generator
{
    public function build(): string { return 'built'; }
}

/**
 * @param Generator $g
 */
function useIt(Generator $g): string
{
    return $g->build();
}
===expect===
