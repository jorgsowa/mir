===description===
Not visible clone method sub class
===file===
<?php
class a {
    private function __clone() {}
}
class b extends a {}

clone new b;
===expect===
InvalidClone@7:1-7:12: cannot clone non-object b
