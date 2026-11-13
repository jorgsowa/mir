===description===
A method template bounded by a class template that the call leaves unbound (a static call)
is checked against the class template's own bound.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Base {}
class Sub extends Base {}
class Other {}

/** @template TResponse of Base */
class TR {
    /**
     * @template R of TResponse
     * @param R $r
     * @return static<R>
     */
    public static function from($r) { return new static(); }
}

function h(Sub $s, Other $o): void {
    TR::from($s);
    TR::from($o);
//  ^^^^^^^^^^^^ InvalidArgument: Argument $r of from() expects 'Base', got 'Other'
}
