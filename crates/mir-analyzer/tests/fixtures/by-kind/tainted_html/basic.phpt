===description===
Basic
===config===
suppress=MixedArrayAccess
===file===
<?php
function test(): void {
    echo $_GET['x'];
//  ^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}
===expect===
