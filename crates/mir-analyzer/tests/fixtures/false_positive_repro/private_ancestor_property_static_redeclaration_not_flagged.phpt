===description===
Child redeclares a same-named property as static against a PRIVATE non-static
ancestor property — legal PHP, a private property isn't inherited so the child's
declaration is a separate, unrelated one.
===file===
<?php
class Base {
    private int $x = 1;
}

class Sub extends Base {
    public static int $x = 2;
}
===expect===
