===description===
A `self` type argument binds to the declaring class, so an override
returning an unrelated class is still a signature mismatch.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
    <MissingThrowsDocblock errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @template T */
interface Cloner {
    /** @return T */
    public function copy(): object;
}

final class Other {}

/** @template-implements Cloner<self> */
final class Order implements Cloner {
    public function copy(): Other { return new Other(); }
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ MethodSignatureMismatch: Method Order::copy() signature mismatch: return type 'Other' is not a subtype of parent 'self(Order)'
}
===expect===
