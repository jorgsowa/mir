===description===
Child redeclares parent property with the same union type in a different
atom order (null|int vs int|null) — semantically identical, no error
===file===
<?php
class A {
    public null|int $x = null;
}

class B extends A {
    public int|null $x = null;
}
===expect===
