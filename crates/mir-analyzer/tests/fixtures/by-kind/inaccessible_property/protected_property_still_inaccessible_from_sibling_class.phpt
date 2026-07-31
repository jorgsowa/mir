===description===
Negative control for the K3 fix: two classes that share a common ancestor
but aren't in an ancestor/descendant relationship with EACH OTHER must
still be denied access to each other's protected properties.
===config===
suppress=MissingConstructor,UnusedParam
===file===
<?php
class Base {
}

class Child extends Base {
    protected int $offset = 0;
}

class Cousin extends Base {
    public function peek(Child $c): int {
        return $c->offset;
    }
}
===expect===
InaccessibleProperty@11:19-11:25: Cannot access property Child::$offset
