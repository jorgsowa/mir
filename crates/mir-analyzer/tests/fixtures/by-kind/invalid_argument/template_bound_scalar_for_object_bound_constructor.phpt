===description===
A scalar passed to a constructor whose class template is bound by a class is an invalid argument.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Element {}

/** @template T of Element */
final class Holder {
    /** @param T $item */
    public function __construct(public $item) {}
}

function run(Element $element): void {
    new Holder($element);
    new Holder('element');
//  ^^^^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $item of Holder::__construct() expects 'Element', got '"element"'
}
