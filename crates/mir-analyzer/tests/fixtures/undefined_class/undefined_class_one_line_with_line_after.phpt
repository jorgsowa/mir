===description===
undefinedClassOneLineWithLineAfter
===file===
<?php
class A {
    public function b() {
        /**
         * @psalm-suppress UndefinedClass
         */
        new B();
        new C();
    }
}
===expect===
UndefinedClass@8:12: Class C does not exist
