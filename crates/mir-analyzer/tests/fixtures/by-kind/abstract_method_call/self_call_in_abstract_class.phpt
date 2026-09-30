===description===
AbstractMethodCall fires when calling an abstract method via self:: within the abstract class itself.
self:: always resolves to the declaring class; if that class declares the method as abstract, the call has no body to execute.
===file===
<?php
abstract class Base {
    abstract public function foo(): void;
    public function bar(): void {
        self::foo();
//      ^^^^^^^^^^^ AbstractMethodCall: Cannot call abstract method Base::foo()
    }
}
===expect===
