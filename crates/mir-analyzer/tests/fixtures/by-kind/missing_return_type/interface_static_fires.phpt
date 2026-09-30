===description===
MissingReturnType fires for a static interface method without a return type hint
===file===
<?php
interface IFoo {
    public static function staticNoReturn();
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ MissingReturnType: Function IFoo::staticNoReturn() has no return type annotation
}
===expect===
