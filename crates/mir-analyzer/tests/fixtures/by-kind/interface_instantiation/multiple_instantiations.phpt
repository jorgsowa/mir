===description===
InterfaceInstantiation fires independently for each interface instantiation in the same file.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
interface Readable {
    public function read(): string;
}

interface Writable {
    public function write(string $data): void;
}

$r = new Readable();
//       ^^^^^^^^ InterfaceInstantiation: Cannot instantiate interface Readable
$w = new Writable();
//       ^^^^^^^^ InterfaceInstantiation: Cannot instantiate interface Writable
===expect===
