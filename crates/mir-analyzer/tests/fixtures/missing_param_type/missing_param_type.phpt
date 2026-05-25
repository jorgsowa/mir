===description===
Missing param type
===file===
<?php
interface foo {
    public function withoutAnyReturnType($s) : void;
}
===expect===
MissingParamType
===ignore===
TODO
