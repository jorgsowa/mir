===description===
A validation guard followed by `or die()` or `|| exit()` clears the checked variable or
array element for the code after it.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function variable(): void {
    $id = $_GET['id'];
    ctype_digit($id) or die();
    echo $id;
}

function element(): void {
    ctype_digit($_GET['id']) || exit();
    echo $_GET['id'];
}

function pattern(): void {
    preg_match('/^\w+$/D', $_GET['k']) or die('bad key');
    echo $_GET['k'];
}

function notDiverging(): void {
    ctype_digit($_GET['id']) or print('not a number');
    echo $_GET['id'];
//  ^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}
