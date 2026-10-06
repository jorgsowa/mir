===description===
InterfaceInstantiation fires when an interface is instantiated inside a class method body.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
interface Storage {
    public function store(string $key, mixed $value): void;
}

class Cache {
    public function clear(): void {
        $s = new Storage();
//               ^^^^^^^ InterfaceInstantiation: Cannot instantiate interface Storage
    }
}
