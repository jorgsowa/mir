===description===
Not visible clone method trait
===file===
<?php
trait a {
    private function __clone() {}
}
class b {
    use a;
}

clone new b;
//<^^^^^^^^^^^ InvalidClone: cannot clone non-object b
