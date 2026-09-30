===description===
`[$_GET['env'] => 'active']` — the attacker controls which KEY exists in
the array, not just its values, but the array-literal taint check only
ever inspected `el.value`, never `el.key`.
===config===
suppress=MixedArrayAccess,MixedArgument
===file===
<?php
function test(): void {
    $config = [$_GET['env'] => 'active'];
    $active = $config['active'] ?? null;
    echo $active;
//  ^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}
===expect===
