===description===
A builtin's result carries its arguments' taint unless the builtin is a
sanitizer or can only return numbers or booleans.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedOperand errorLevel="suppress"/>
    <PossiblyFalseOperand errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function passThrough(): void {
    echo substr($_GET['a'], 0, 10);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo urldecode($_GET['a']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo base64_decode($_GET['a']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo html_entity_decode(htmlspecialchars($_GET['a']));
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    $ids = array_values($_GET);
    echo $ids[0];
//  ^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function safeResults(): void {
    echo strlen($_GET['a']);
    echo count($_GET);
    echo intval($_GET['a']);
    echo filter_var($_GET['a'], FILTER_VALIDATE_INT);
    echo filter_var($_GET['a'], FILTER_SANITIZE_FULL_SPECIAL_CHARS);
    echo date('Y-m-d');
}

function requestSources(mysqli $db): void {
    echo filter_input(INPUT_GET, 'q');
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo filter_input(INPUT_GET, 'id', FILTER_VALIDATE_INT);
    $body = file_get_contents('php://input');
    echo $body;
//  ^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    $escaped = filter_input(INPUT_POST, 'name', FILTER_SANITIZE_SPECIAL_CHARS);
    echo $escaped;
    mysqli_query($db, "SELECT * FROM t WHERE name = '" . $escaped . "'");
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedSql: Tainted SQL query — possible SQL injection
}
