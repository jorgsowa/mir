===description===
Gap #10: concrete class extends abstract but implements none of the abstract methods
===file===
<?php
abstract class AbstractBase {
    abstract public function compute(): int;
    abstract public function render(): string;
}
class ConcreteChild extends AbstractBase {
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UnimplementedAbstractMethod: Class ConcreteChild must implement abstract method compute()
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UnimplementedAbstractMethod: Class ConcreteChild must implement abstract method render()
    // implements neither compute() nor render()
}
===expect===
