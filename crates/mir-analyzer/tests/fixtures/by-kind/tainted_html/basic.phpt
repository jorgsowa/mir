===description===
Basic
===config===
suppress=MixedArrayAccess
===file===
<?php
function test(): void {
    echo $_GET['x'];
}
===expect===
TaintedHtml@3:4-3:20: Tainted HTML output — possible XSS
