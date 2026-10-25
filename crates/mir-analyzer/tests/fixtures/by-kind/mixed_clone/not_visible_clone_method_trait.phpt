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
//<^^^^^^^^^^^ InvalidClone: Call to private method a::__clone() from global scope
