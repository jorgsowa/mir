===description===
An argument that reaches a sink inside the callee is reported at the call,
through any number of wrappers, only for the kinds the callee doesn't
sanitize, and for named, variadic and spread arguments.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedOperand errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function render(string $html): void { echo $html; }
function renderTwice(string $html): void { render($html); render($html); }
function renderEscaped(string $html): void { render(htmlspecialchars($html)); }
function lookup(mysqli $db, string $name): void {
    echo htmlspecialchars($name);
    mysqli_query($db, "SELECT * FROM t WHERE name = '" . $name . "'");
}
function renderAll(string $title, string ...$lines): void {
    echo htmlspecialchars($title);
    foreach ($lines as $line) {
        echo $line;
    }
}
function renderInt(int $n): void { echo $n; }

function direct(): void {
    render($_GET['x']);
//  ^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    renderTwice($_GET['x']);
//  ^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    render(html: $_GET['x']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    renderAll('title', 'a', $_GET['x']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    renderAll(...$_GET['lines']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function notReported(): void {
    render(htmlspecialchars($_GET['x']));
    renderEscaped($_GET['x']);
    renderAll($_GET['title']);
    renderInt((int) $_GET['n']);
    render('literal');
}

function onlyTheUnsanitizedKind(mysqli $db): void {
    lookup($db, $_GET['name']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedSql: Tainted SQL query — possible SQL injection
}
