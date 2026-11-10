===description===
A native `resource` property type is a class name to PHP, not an undefined class: NativeResourceType.
===config===
<mir>
  <issueHandlers>
    <MissingConstructor errorLevel="suppress"/>
    <UnusedClass errorLevel="suppress"/>
    <UnusedProperty errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class HandleBox {
    public resource $handle;
//         ^^^^^^^^ NativeResourceType: 'resource' is not a native type; PHP reads it as class 'resource', which no resource satisfies
}
