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
===expect===
InvalidClone@9:1-9:12: cannot clone non-object b
