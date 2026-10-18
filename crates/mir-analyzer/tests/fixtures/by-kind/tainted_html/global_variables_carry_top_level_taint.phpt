===description===
A function importing a global with `global $name`, or reading
`$GLOBALS['name']`, sees the taint the file's top-level code left in it.
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
$search = trim((string) $_GET['q']);
$title = htmlspecialchars((string) $_GET['title']);
$site = 'Example';

function header_bar(): void {
    global $search, $title, $site;
    echo $search;
//  ^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo $title;
    echo $site;
}

function footer(): void {
    echo $GLOBALS['search'];
//  ^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo $GLOBALS['site'];
}

header_bar();
footer();
