===description===
Missing return type
===file===
<?php
interface foo {
    public function withoutAnyReturnType();
}
===expect===
MissingReturnType
===ignore===
TODO
