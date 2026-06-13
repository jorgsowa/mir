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
TaintedHtml@3:5-3:21: Tainted HTML output — possible XSS
