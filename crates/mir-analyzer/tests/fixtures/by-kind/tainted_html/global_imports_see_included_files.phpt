===description===
A global imported with `global $name` or read through `$GLOBALS` carries the
taint the top-level code of any file joined to this one by static includes
leaves in it: the file including it, and files that file includes.
Top-level code of an unrelated file doesn't count.
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
===file:index.php===
<?php
$query = trim((string) $_GET['q']);
$title = 'Home';
require __DIR__ . '/setup.php';
require 'view.php';
render_page();
===file:setup.php===
<?php
$user = (string) $_COOKIE['user'];
$escapedUser = htmlspecialchars($user);
===file:view.php===
<?php
function render_page(): void {
    global $query, $title, $user, $escapedUser, $stray;
    echo $query;
//  ^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo $title;
    echo $user;
//  ^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo $escapedUser;
    echo $stray;
    echo $GLOBALS['query'];
//  ^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}
===file:unrelated.php===
<?php
$stray = (string) $_GET['stray'];
