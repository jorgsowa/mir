===description===
variable from superglobal
===file===
<?php
function test(): void {
    $name = $_POST['name'];
    echo $name;
}
===expect===
TaintedHtml: Tainted HTML output — possible XSS
===ignore===
TODO
