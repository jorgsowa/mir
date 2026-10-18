===description===
A caller's taint report appears once the callee in another file stops
escaping its argument.
===file:Lib.php===
<?php
function label(string $v): string { return htmlspecialchars($v); }
===file:Use.php===
<?php
function run(): void { echo label((string) $_GET['x']); }
===expect===
Use.php: MixedArrayAccess@2:43-2:53: Array access on mixed type
===edit:Lib.php===
<?php
function label(string $v): string { return '<b>' . $v . '</b>'; }
===expect===
Use.php: MixedArrayAccess@2:43-2:53: Array access on mixed type
Use.php: TaintedHtml@2:23-2:55: Tainted HTML output — possible XSS
