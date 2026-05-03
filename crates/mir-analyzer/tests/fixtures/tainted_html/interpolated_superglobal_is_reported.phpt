===description===
interpolated superglobal is reported
===file===
<?php
function render(): void {
    echo "Hello {$_GET['name']}";
}
===expect===
TaintedHtml: Tainted HTML output — possible XSS
===ignore===
TODO
