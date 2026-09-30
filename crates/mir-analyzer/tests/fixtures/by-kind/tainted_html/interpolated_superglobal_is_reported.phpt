===description===
interpolated superglobal is reported
===config===
suppress=MixedArrayAccess
===file===
<?php
function render(): void {
    echo "Hello {$_GET['name']}";
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}
===expect===
