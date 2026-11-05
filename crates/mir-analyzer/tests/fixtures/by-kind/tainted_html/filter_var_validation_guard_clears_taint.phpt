===description===
A passing `filter_var()` validation proves the value's characters. A sanitizing filter, or
options that can supply a default, prove nothing about the variable.
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
function notFalse(): void {
    $x = $_GET['x'];
    if (filter_var($x, FILTER_VALIDATE_INT) !== false) {
        echo $x;
    }
}

function truthy(): void {
    $x = $_GET['x'];
    if (filter_var($x, FILTER_VALIDATE_IP)) {
        echo $x;
    }
}

function exits(): void {
    if (filter_var($_GET['x'], FILTER_VALIDATE_FLOAT) === false) {
        exit;
    }
    echo $_GET['x'];
}

function failingPath(): void {
    $x = $_GET['x'];
    if (filter_var($x, FILTER_VALIDATE_INT) === false) {
        echo $x;
//      ^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}

function sanitizingFilter(): void {
    $x = $_GET['x'];
    if (filter_var($x, FILTER_SANITIZE_NUMBER_INT)) {
        echo $x;
//      ^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}

function withDefault(): void {
    $x = $_GET['x'];
    if (filter_var($x, FILTER_VALIDATE_INT, ['options' => ['default' => 1]])) {
        echo $x;
//      ^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}
