===description===
InterfaceInstantiation does NOT fire when instantiating a concrete class.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class FileLogger implements \Countable {
    public function count(): int { return 0; }
}

$l = new FileLogger();
