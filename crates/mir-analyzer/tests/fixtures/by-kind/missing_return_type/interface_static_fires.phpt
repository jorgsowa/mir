===description===
MissingReturnType fires for a static interface method without a return type hint
===file===
<?php
interface IFoo {
    public static function staticNoReturn();
}
===expect===
MissingReturnType@3:4-3:44: Function IFoo::staticNoReturn() has no return type annotation
