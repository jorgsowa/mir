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
TaintedHtml@4:4-4:15: Tainted HTML output — possible XSS
