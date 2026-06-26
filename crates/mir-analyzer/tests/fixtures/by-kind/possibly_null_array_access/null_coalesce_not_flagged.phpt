===description===
Null coalesce assignment resolves the union to a concrete array — no PossiblyNullArrayAccess after ??
===file===
<?php
/**
 * @param array<string>|null $arr
 */
function test(?array $arr): void {
    $arr = $arr ?? [];
    echo $arr[0];
}
===expect===
