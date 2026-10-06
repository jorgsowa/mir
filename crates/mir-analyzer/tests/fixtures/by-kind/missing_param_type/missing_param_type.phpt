===description===
Missing param type
===file===
<?php
interface foo {
    public function withoutAnyReturnType($s) : void;
//                                       ^^ MissingParamType: Parameter $s of foo::withoutAnyReturnType() has no type annotation
}
