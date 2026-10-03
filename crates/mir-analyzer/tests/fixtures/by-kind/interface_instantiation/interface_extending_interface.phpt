===description===
InterfaceInstantiation fires for an interface that extends another interface.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
interface Loggable {
    public function log(): void;
}

interface StructuredLoggable extends Loggable {
    public function structuredLog(): array;
}

$l = new StructuredLoggable();
//       ^^^^^^^^^^^^^^^^^^ InterfaceInstantiation: Cannot instantiate interface StructuredLoggable
===expect===
