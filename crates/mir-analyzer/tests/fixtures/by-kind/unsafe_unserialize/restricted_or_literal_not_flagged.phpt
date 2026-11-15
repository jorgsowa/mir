===description===
Restricted classes, literal payloads, and tainted payloads (reported as
`TaintedUnserialize` instead) don't raise `UnsafeUnserialize`.
===config===
<mir>
  <issueHandlers>
    <UnsafeUnserialize errorLevel="warning"/>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function load(string $stored): void {
    unserialize($stored, ['allowed_classes' => false]);
    unserialize($stored, ['allowed_classes' => [DateTime::class]]);
    unserialize('a:0:{}');
    unserialize($_COOKIE['cart']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedUnserialize: Tainted unserialize() payload — possible object injection
}
