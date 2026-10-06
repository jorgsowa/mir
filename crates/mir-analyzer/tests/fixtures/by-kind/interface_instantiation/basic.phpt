===description===
InterfaceInstantiation fires when trying to instantiate an interface.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
interface Countable {
    public function count(): int;
}

$c = new Countable();
//       ^^^^^^^^^ InterfaceInstantiation: Cannot instantiate interface Countable
