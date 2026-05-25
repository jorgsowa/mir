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
    }
}
===expect===
UndefinedClass@8:13: Class C does not exist
