===description===
A sanitizer only neutralizes the sink kind it escapes for: HTML-escaped
input is still dangerous in SQL or a shell command, and SQL-escaped input is
still dangerous in HTML.
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
function htmlEscaped(mysqli $db): void {
    $name = htmlspecialchars($_GET['name']);
    echo $name;
    mysqli_query($db, "SELECT * FROM users WHERE name = '" . $name . "'");
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedSql: Tainted SQL query — possible SQL injection
    system('grep ' . $name . ' /var/log/app.log');
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedShell: Tainted shell command — possible command injection
}

function sqlEscaped(mysqli $db, PDO $pdo): void {
    $name = mysqli_real_escape_string($db, $_GET['name']);
    mysqli_query($db, "SELECT * FROM users WHERE name = '" . $name . "'");
    echo $name;
//  ^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    $pdo->query('SELECT * FROM users WHERE name = ' . $pdo->quote($_GET['name']));
}

function shellEscaped(): void {
    $file = escapeshellarg($_GET['file']);
    system('cat ' . $file);
    echo $file;
//  ^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function hashed(mysqli $db): void {
    $token = md5($_GET['token']);
    echo $token;
    mysqli_query($db, "SELECT * FROM sessions WHERE token = '" . $token . "'");
}
