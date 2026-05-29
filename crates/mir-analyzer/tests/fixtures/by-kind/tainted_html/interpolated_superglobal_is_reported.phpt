===description===
interpolated superglobal is reported
===file===
<?php
function render(): void {
    echo "Hello {$_GET['name']}";
}
===expect===
TaintedHtml@3:5-3:34: Tainted HTML output — possible XSS
