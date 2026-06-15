===description===
Missing return type
===file===
<?php
interface foo {
    public function withoutAnyReturnType();
}
===expect===
MissingReturnType@3:4-3:43: Function foo::withoutAnyReturnType() has no return type annotation
