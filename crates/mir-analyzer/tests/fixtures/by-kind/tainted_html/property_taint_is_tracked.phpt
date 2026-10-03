===description===
Taint via a single-hop instance property assignment must be tracked —
assigning a tainted value to $obj->prop and later reading $obj->prop was
previously never recognized as tainted (is_expr_tainted had no
PropertyAccess arm at all).
===config===
<mir>
  <issueHandlers>
    <MixedAssignment errorLevel="suppress"/>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MissingPropertyType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Box {
    public $value;
}
function test(): void {
    $b = new Box();
    $b->value = $_GET['x'];
    echo $b->value;
//  ^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}
===expect===
