===description===
$arr['k'] = $tainted; then reading $arr['k'] back had no taint
propagation at all -- the assign-target match had no ArrayAccess arm,
unlike the Variable/PropertyAccess/Array(destructuring) arms right
above it.
===config===
<mir>
  <issueHandlers>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function test(): void {
    $arr = [];
    $arr['x'] = $_GET['name'];
    echo $arr['x'];
//  ^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}
