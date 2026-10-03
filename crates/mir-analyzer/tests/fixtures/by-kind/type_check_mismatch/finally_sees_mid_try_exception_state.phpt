===description===
Inside `finally`, a variable reassigned partway through the try body isn't
narrowed to its final value alone — an exception could have been thrown
before the try body completed, same conservative pre/post merge already
used to seed catch blocks
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class A {}
class B {}
function risky(): void {}
function f(): void {
    $x = null;
    try {
        $x = new A();
        risky();
        $x = new B();
    } finally {
        /** @mir-check $x is B|null */
        echo get_class($x);
//                     ^^ PossiblyNullArgument: Argument $object of get_class() might be null
    }
}
===expect===
