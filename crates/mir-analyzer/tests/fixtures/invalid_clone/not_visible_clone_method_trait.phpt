===description===
notVisibleCloneMethodTrait
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
InvalidClone
===ignore===
TODO
