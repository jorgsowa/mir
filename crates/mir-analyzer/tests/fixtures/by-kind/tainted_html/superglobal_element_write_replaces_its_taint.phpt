===description===
Writing a clean value to a superglobal element clears that element only.
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
function written(): void {
    $_POST['n'] = (int) $_POST['n'];
    echo $_POST['n'];
    $_GET['a'] = 'c';
    echo $_GET['a'];
}

function otherKeyStaysTainted(): void {
    $_GET['a'] = 'c';
    echo $_GET['b'];
//  ^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function taintedWrite(): void {
    $_GET['a'] = $_POST['b'];
    echo $_GET['a'];
//  ^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function branch(bool $c): void {
    if ($c) {
        $_GET['a'] = 'c';
    }
    echo $_GET['a'];
//  ^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}
