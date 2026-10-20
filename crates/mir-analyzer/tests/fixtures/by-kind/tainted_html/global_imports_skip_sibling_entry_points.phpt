===description===
Two entry points that include the same bootstrap file don't see each other's
globals, only the ones the bootstrap file sets.
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
===file:search.php===
<?php
$title = (string) $_GET['t'];
require 'bootstrap.php';
===file:about.php===
<?php
require 'bootstrap.php';
function about_page(): void {
    global $title, $fromBootstrap;
    echo $title;
    echo $fromBootstrap;
//  ^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}
about_page();
===file:bootstrap.php===
<?php
$fromBootstrap = (string) $_GET['b'];
