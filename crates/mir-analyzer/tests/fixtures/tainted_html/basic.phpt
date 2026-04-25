===file===
<?php
function test(): void {
    echo $_GET['x'];
}
===expect===
TaintedHtml: Tainted HTML output — possible XSS
