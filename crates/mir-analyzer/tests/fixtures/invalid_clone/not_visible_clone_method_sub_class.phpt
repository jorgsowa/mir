===description===
notVisibleCloneMethodSubClass
===file===
<?php
class a {
    private function __clone() {}
}
class b extends a {}

clone new b;
===expect===
InvalidClone
===ignore===
TODO
