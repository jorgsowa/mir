===description===
print($_GET['x']) is an HTML sink, same as echo $_GET['x'].
===config===
suppress=MixedArrayAccess
===file===
<?php
function test(): void {
    print($_GET['x']);
}
===expect===
TaintedHtml@3:4-3:21: Tainted HTML output — possible XSS
