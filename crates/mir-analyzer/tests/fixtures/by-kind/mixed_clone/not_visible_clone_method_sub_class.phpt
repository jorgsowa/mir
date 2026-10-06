===description===
Not visible clone method sub class
===file===
<?php
class a {
    private function __clone() {}
}
class b extends a {}

clone new b;
//<^^^^^^^^^^^ InvalidClone: cannot clone non-object b
