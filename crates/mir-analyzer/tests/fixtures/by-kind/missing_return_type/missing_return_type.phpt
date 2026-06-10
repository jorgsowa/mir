===description===
Missing return type
===file===
<?php
interface foo {
    public function withoutAnyReturnType();
}
===expect===
MissingReturnType@3:5-3:44: Function foo::withoutAnyReturnType() has no return type annotation
