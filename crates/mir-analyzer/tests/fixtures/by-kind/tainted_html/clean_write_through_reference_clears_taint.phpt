===description===
A clean write to either side of a `=&` binding clears the other side's taint.
===config===
<mir>
  <issueHandlers>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function f(): void {
    $b = $_GET['x'];
    $a = &$b;
    $b = 'c';
    echo $a;
}
function g(): void {
    $b = $_GET['x'];
    $a = &$b;
    $a = 'c';
    echo $b;
}
