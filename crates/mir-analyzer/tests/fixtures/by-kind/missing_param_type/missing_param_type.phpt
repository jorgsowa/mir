===description===
Missing param type
===file===
<?php
interface foo {
    public function withoutAnyReturnType($s) : void;
}
===expect===
MissingParamType@3:41-3:43: Parameter $s of foo::withoutAnyReturnType() has no type annotation
