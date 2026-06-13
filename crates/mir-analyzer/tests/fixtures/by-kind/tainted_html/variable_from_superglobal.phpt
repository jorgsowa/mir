===description===
variable from superglobal
===config===
suppress=MixedArrayAccess,MixedAssignment
===file===
<?php
function test(): void {
    $name = $_POST['name'];
    echo $name;
}
===expect===
TaintedHtml@4:5-4:16: Tainted HTML output — possible XSS
