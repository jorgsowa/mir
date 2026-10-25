===description===
A concrete class or enum must implement the abstract property hooks it inherits. A backed property implements both hooks; a virtual one (no hook touches `$this->prop`) only those it declares.
===config===
<mir>
  <phpVersion>8.4</phpVersion>
</mir>
===file===
<?php
interface HasId {
    public int $id { get; }
}

interface Named {
    public string $name { get; set; }
}

abstract class Shape {
    abstract public float $area { get; }
}

class Missing implements HasId, Named {}
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ CompileError: Class Missing contains 3 abstract methods and must therefore be declared abstract or implement the remaining methods (HasId::$id::get, Named::$name::get, Named::$name::set) [unimplemented-property-hook]

class Square extends Shape {}
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ CompileError: Class Square contains 1 abstract method and must therefore be declared abstract or implement the remaining methods (Shape::$area::get) [unimplemented-property-hook]

enum Status implements HasId {
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ CompileError: Enum Status must implement 1 abstract private method (HasId::$id::get) [unimplemented-property-hook]
    case Active;
}

class Backed implements Named {
    public string $name { get => ucfirst($this->name); }
}

class Virtual implements Named {
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ CompileError: Class Virtual contains 1 abstract method and must therefore be declared abstract or implement the remaining methods (Named::$name::set) [unimplemented-property-hook]
    public string $name { get => 'fixed'; }
}

class Plain implements HasId, Named {
    public function __construct(public int $id, public string $name) {}
}

abstract class Partial implements HasId {}
