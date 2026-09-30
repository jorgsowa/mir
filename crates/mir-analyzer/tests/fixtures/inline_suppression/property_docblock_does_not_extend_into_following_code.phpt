===description===
The body-extension fix is scoped to function/method declarations only — a
suppression above a non-function declaration (a property) must keep
covering just its own declaration line, same as before this fix.
===file===
<?php
class C {
//<^^^^^^^^^ MissingConstructor: Class C has uninitialized properties but no constructor
    /** @psalm-suppress UndefinedClass */
    private NoSuchClassA $prop;

    public function m(): NoSuchClassB {
//                       ^^^^^^^^^^^^ UndefinedClass: Class NoSuchClassB does not exist
        return new NoSuchClassB();
//                 ^^^^^^^^^^^^ UndefinedClass: Class NoSuchClassB does not exist
    }
}
===expect===
