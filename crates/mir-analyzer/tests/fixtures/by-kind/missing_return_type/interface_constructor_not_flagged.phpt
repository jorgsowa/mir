===description===
MissingReturnType does not fire for an interface's __construct or __destruct: PHP forbids a return type on them.
===file===
<?php
interface Creatable {
    public function __construct(int $id);
    public function __destruct();
    public function name();
//  ^^^^^^^^^^^^^^^^^^^^^^^ MissingReturnType: Function Creatable::name() has no return type annotation
}
