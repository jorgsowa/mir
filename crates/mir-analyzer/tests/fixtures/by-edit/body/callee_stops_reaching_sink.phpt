===description===
A call reported because the callee echoed its argument is no longer
reported once the callee in another file escapes it.
===file:Lib.php===
<?php
function show(string $v): void { echo $v; }
===file:Use.php===
<?php
function run(): void { show((string) $_GET['x']); }
===expect===
Use.php: MixedArrayAccess@2:37-2:47: Array access on mixed type
Use.php: TaintedHtml@2:23-2:48: Tainted HTML output — possible XSS
===edit:Lib.php===
<?php
function show(string $v): void { echo htmlspecialchars($v); }
===expect===
Use.php: MixedArrayAccess@2:37-2:47: Array access on mixed type
