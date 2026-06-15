===description===
interpolated superglobal is reported
===config===
suppress=MixedArrayAccess
===file===
<?php
function render(): void {
    echo "Hello {$_GET['name']}";
}
===expect===
TaintedHtml@3:4-3:33: Tainted HTML output — possible XSS
