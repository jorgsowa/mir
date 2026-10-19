===description===
`$x = &$a[$k]` binds a reference and auto-vivifies a null base, so a
nullable array base is not a PossiblyNullArrayAccess (also when nested).
===config===
<mir>
  <issueHandlers>
    <MixedAssignment errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedArrayOffset errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param array<string, mixed>|null $a */
function walk(?array $a, string $k): void {
    $r = &$a[$k];
    $r = 1;
}

function narrowed(mixed $a, string $k): void {
    if (is_array($a) || $a === null) {
        $a = &$a[$k];
    }
}

/** @param array<string, array<string, int>|null>|null $a */
function nested(?array $a): void {
    $r = &$a['x']['y'];
    $r = 1;
}

function property(object $o): void {
    $r = &$o->items['k'];
    $r = 1;
}
