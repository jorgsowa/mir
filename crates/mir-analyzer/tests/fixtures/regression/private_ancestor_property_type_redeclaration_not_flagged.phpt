===description===
Child redeclares a same-named property with a different native type against a
PRIVATE ancestor property — legal PHP, a private property establishes no type
contract for a subclass since it isn't inherited.
===file===
<?php
class Base {
    private int $x = 1;
}

class Sub extends Base {
    public string $x = 'hello';
}
===expect===
