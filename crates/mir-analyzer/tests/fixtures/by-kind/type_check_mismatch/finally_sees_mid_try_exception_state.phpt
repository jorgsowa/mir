===description===
Inside `finally`, a variable reassigned partway through the try body may hold
any value it had after a statement of the try body
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
        /** @mir-check $x is B|A|null */
        echo get_class($x);
//                     ^^ PossiblyNullArgument: Argument $object of get_class() might be null
    }
}
