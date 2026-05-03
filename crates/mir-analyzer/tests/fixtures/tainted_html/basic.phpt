===description===
basic
===file===
<?php
function test(): void {
    echo $_GET['x'];
}
===expect===
TaintedHtml@3:4: Tainted HTML output — possible XSS
===ignore===
TODO
