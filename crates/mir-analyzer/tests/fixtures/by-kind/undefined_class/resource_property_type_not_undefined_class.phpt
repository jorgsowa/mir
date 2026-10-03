===description===
Legacy `resource` property types are not undefined classes.
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
}
===expect===
