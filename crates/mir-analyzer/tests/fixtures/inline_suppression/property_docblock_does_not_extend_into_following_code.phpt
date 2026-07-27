===description===
The body-extension fix is scoped to function/method declarations only — a
suppression above a non-function declaration (a property) must keep
covering just its own declaration line, same as before this fix.
===file===
<?php
class C {
    /** @psalm-suppress UndefinedClass */
    private NoSuchClassA $prop;

    public function m(): NoSuchClassB {
        return new NoSuchClassB();
    }
}
===expect===
MissingConstructor@2:0-2:9: Class C has uninitialized properties but no constructor
UndefinedClass@6:25-6:37: Class NoSuchClassB does not exist
UndefinedClass@7:19-7:31: Class NoSuchClassB does not exist
