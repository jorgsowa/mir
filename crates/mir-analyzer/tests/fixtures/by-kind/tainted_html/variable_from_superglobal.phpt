===description===
variable from superglobal
===file===
<?php
function test(): void {
    $name = $_POST['name'];
    echo $name;
}
===expect===
TaintedHtml@4:5: Tainted HTML output — possible XSS
