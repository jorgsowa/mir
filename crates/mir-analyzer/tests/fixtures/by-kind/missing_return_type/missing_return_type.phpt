===description===
Missing return type
===file===
<?php
interface foo {
    public function withoutAnyReturnType();
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ MissingReturnType: Function foo::withoutAnyReturnType() has no return type annotation
}
===expect===
