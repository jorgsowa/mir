===description===
An object passes `object{…}&stdClass` when it is a stdClass, as `(object) [...]` is.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <MissingParamType errorLevel="suppress"/>
    <MissingReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace App;

final class Point
{
    public int $foo = 1;
}

/** @param object{foo: int}&\stdClass $shape */
function takesWritableObjectShape(object $shape): void {}

takesWritableObjectShape((object) ['foo' => 1]);
takesWritableObjectShape(new \stdClass());
takesWritableObjectShape(new Point());
//                       ^^^^^^^^^^^ InvalidArgument: Argument $shape of takesWritableObjectShape() expects 'object{foo: int}&stdClass', got 'App\Point'
