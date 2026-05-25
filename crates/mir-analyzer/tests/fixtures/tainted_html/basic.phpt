===description===
Basic
===file===
<?php
function test(): void {
    echo $_GET['x'];
}
===expect===
TaintedHtml@3:5: Tainted HTML output — possible XSS
