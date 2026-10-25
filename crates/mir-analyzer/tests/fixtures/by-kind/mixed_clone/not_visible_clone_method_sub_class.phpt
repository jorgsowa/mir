===description===
Not visible clone method sub class
===file===
<?php
class a {
    private function __clone() {}
}
class b extends a {}

clone new b;
//<^^^^^^^^^^^ InvalidClone: Call to private method a::__clone() from global scope
