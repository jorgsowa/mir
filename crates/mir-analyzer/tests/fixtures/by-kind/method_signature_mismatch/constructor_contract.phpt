===description===
A constructor answers to an abstract or interface constructor anywhere up the chain, and to a final one; an ordinary parent constructor imposes no signature or visibility.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
interface Creatable {
    public function __construct(int $id);
}

abstract class Model implements Creatable {}

class User extends Model {
    public function __construct(string $id) {}
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ MethodSignatureMismatch: Method User::__construct() signature mismatch: parameter $id type 'string' is incompatible with parent type 'int'
}

abstract class Shape {
    abstract protected function __construct(int $sides);
}

class Square extends Shape {
    private function __construct(int $sides) {}
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ OverriddenMethodAccess: Access level to Square::__construct() must be protected (as in class Shape) or weaker
}

class Polygon extends Shape {
    public function __construct(int $sides) {}
}

class Triangle extends Polygon {
    public function __construct() {}
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ MethodSignatureMismatch: Method Triangle::__construct() signature mismatch: method has fewer parameters (0) than parent Shape::__construct() (1)
}

class Locked {
    final public function __construct() {}
}

class Unlocked extends Locked {
    public function __construct() {}
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ FinalMethodOverridden: Method Unlocked::__construct() cannot override final method from Locked
}

class Plain {
    public function __construct(int $a) {}
}

class Loose extends Plain {
    private function __construct(string $b, $c) {}
}
