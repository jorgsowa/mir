===description===
Undefined class one line with line after
===file===
<?php
class A {
    public function b() {
        /**
         * @suppress UndefinedClass
         */
        new B();
        new C();
//          ^ UndefinedClass: Class C does not exist
    }
}
===expect===
