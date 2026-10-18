===description===
`@psalm-taint-escape` removes kinds from a function's result,
`@psalm-taint-unescape` restores them, and `@psalm-flow` declares which
parameters flow into the result of a method without a body.
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
/** @psalm-taint-escape html */
function myEscape(string $v): string { return str_replace(['<', '>'], '', $v); }

/** @taint-escape sql */
function quoteSql(string $v): string { return addcslashes($v, "'"); }

/** @psalm-taint-unescape html */
function decode(string $v): string { return strtr($v, ['&lt;' => '<', '&gt;' => '>']); }

interface Templates {
    /** @psalm-flow ($name) -> return */
    public function greeting(string $name, string $lang): string;

    /** @psalm-taint-escape html */
    public function escape(string $v): string;
}

function useThem(Templates $t, mysqli $db): void {
    echo myEscape($_GET['x']);
    mysqli_query($db, 'SELECT ' . myEscape($_GET['x']));
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedSql: Tainted SQL query — possible SQL injection
    mysqli_query($db, "SELECT '" . quoteSql($_GET['x']) . "'");
    echo decode(htmlspecialchars($_GET['x']));
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo $t->greeting($_GET['x'], 'en');
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo $t->greeting('Ann', $_GET['lang']);
    echo $t->escape($_GET['x']);
    mysqli_query($db, 'SELECT ' . $t->escape($_GET['x']));
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedSql: Tainted SQL query — possible SQL injection
}
