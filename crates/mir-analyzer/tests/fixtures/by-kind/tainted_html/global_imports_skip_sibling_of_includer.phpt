===description===
Globals flow along an include chain: from every ancestor, what it includes and
across a cycle. A file that only shares an included file stays apart.
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
===file:front.php===
<?php
$fromFront = (string) $_GET['f'];
require 'middle.php';
===file:middle.php===
<?php
$fromMiddle = (string) $_GET['m'];
require 'leaf.php';
require 'front.php';
===file:leaf.php===
<?php
$fromLeaf = (string) $_GET['l'];
require 'shared.php';
function show_leaf(): void {
    global $fromFront, $fromMiddle, $fromLeaf, $fromShared, $fromOther;
    echo $fromFront;
//  ^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo $fromMiddle;
//  ^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo $fromLeaf;
//  ^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo $fromShared;
//  ^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo $fromOther;
}
===file:other.php===
<?php
$fromOther = (string) $_GET['o'];
require 'shared.php';
===file:shared.php===
<?php
$fromShared = (string) $_GET['s'];
