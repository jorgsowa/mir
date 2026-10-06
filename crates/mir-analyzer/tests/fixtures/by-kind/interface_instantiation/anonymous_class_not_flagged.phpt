===description===
InterfaceInstantiation does NOT fire for an anonymous class that implements an interface.
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

$c = new class() implements Countable {
    public function count(): int { return 0; }
};
