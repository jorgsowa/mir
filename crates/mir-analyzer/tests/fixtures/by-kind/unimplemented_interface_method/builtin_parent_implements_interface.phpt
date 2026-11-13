===description===
A subclass of a built-in class isn't asked for interface methods the built-in provides,
even one its stub marks as added in a newer PHP version.
===config===
<mir>
  <phpVersion>8.2</phpVersion>
</mir>
===file===
<?php
class Scope extends \SplObjectStorage {}

interface Runnable { public function run(): void; }
class Task extends \ArrayObject implements Runnable {}
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UnimplementedInterfaceMethod: Class Task must implement Runnable::run() from interface
