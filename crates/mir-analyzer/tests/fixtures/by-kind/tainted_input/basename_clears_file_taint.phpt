===description===
`basename()` strips directory components, so its result no longer carries path
traversal taint (the php.net upload pattern). Other sinks still see the value.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedOperand errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function upload(string $dir): void {
    move_uploaded_file($_FILES['f']['tmp_name'], $dir . '/' . basename($_FILES['f']['name']));
}

function readCached(): string {
    return file_get_contents(__DIR__ . '/cache/' . basename($_GET['q']));
}

function viaVariable(): void {
    $name = basename($_POST['name']);
    file_put_contents('/var/log/' . $name, 'x');
    unlink('/var/log/' . $name);
}

function nested(): void {
    file_get_contents(basename(trim($_GET['p'])));
    file_get_contents(trim(basename($_GET['p'])));
}

function suffixArgument(): void {
    file_get_contents(basename($_GET['p'], '.php'));
}

function throughHelper(string $p): string {
    return basename($p);
}

function helperCaller(): void {
    file_get_contents('/srv/' . throughHelper($_GET['p']));
}

function mappedOverArray(): void {
    foreach (array_map('basename', $_GET['files']) as $f) {
        file_get_contents('/srv/' . $f);
    }
}

function stillTaintedForOtherKinds(): void {
    echo basename($_GET['q']);
    $mysqli = new mysqli();
    $mysqli->query('SELECT * FROM t WHERE n = ' . basename($_GET['q']));
    system(basename($_GET['q']));
}

function unrelatedFunctionKeepsTaint(): void {
    file_get_contents('/srv/' . dirname($_GET['q']));
    file_get_contents('/srv/' . pathinfo($_GET['q'], PATHINFO_DIRNAME));
    file_get_contents('/srv/' . $_GET['q']);
}

function onlyBasenameOfOneOperand(): void {
    file_get_contents(basename($_GET['a']) . $_GET['b']);
}
===expect===
TaintedHtml@40:4-40:30: Tainted HTML output — possible XSS
TaintedSql@42:4-42:71: Tainted SQL query — possible SQL injection
TaintedShell@43:4-43:32: Tainted shell command — possible command injection
TaintedInput@47:4-47:52: Tainted input reaching sink 'file'
TaintedInput@48:4-48:71: Tainted input reaching sink 'file'
TaintedInput@49:4-49:43: Tainted input reaching sink 'file'
TaintedInput@53:4-53:56: Tainted input reaching sink 'file'
