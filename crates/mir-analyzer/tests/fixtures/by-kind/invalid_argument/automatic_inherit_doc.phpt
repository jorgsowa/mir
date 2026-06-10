===description===
Automatic inherit doc
===ignore===
TODO
===file===
<?php
class Y {
    /**
     * @param string[] $arr
     */
    public function boo(array $arr) : void {}
}

class X extends Y {
    public function boo(array $arr) : void {}
}

(new X())->boo([1, 2]);
===expect===
