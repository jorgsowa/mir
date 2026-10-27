===description===
Before property hooks `parent::$x::get()` reads `parent::$x`, so a class without a parent cannot use it; traits are not checked.
===config===
<mir>
  <phpVersion>8.3</phpVersion>
  <issueHandlers>
    <UndefinedProperty errorLevel="suppress"/>
    <MixedMethodCall errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class P {
    function f(): void { parent::$x::get(); }
//                       ^^^^^^ ParentNotFound: Cannot use parent:: when current class has no parent
}
class Q extends P {
    function f(): void { parent::$x::get(); }
}
trait T {
    function f(): void { parent::$x::get(); }
}
echo get_class(new class {
    function f(): void { parent::$x::get(); }
//                       ^^^^^^ ParentNotFound: Cannot use parent:: when current class has no parent
});
