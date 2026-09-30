===description===
Parent static call
===file===
<?php
class A {
    /** @return void */
    public function foo(){}
}

class B extends A {
    /** @return void */
    public static function bar(){
        parent::foo();
//      ^^^^^^^^^^^^^ NonStaticSelfCall: Non-static method A::foo() cannot be called statically
    }
}
===expect===
